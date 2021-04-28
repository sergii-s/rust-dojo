// use actix::prelude::*;
use std::borrow::Borrow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::Mutex;
use tokio::sync::oneshot;
use tokio::sync::oneshot::{Receiver, Sender};
use std::ops::Deref;
use tokio::sync::oneshot::error::RecvError;
use actix::{Message, Recipient, Actor, Context, Handler, WrapFuture, ActorFutureExt, SpawnHandle, ContextFutureSpawner, ActorContext, AsyncContext, Addr, MailboxError, Running, ResponseFuture, ActorState};
use std::io::Error;
use actix::dev::SendError;
use core::mem;
use std::pin::Pin;
use std::future::Future;

// ###################################### to file - MessageSenderActor

#[derive(Message)]
#[rtype(result = "()")]
struct ShutDown;

const BATCH_SIZE: usize = 5;
const BATCH_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Message)]
#[rtype(result = "()")]
struct PubSubSerializedMessage(Vec<u8>);

struct MessageBatchingActor {
    buffer: Vec<PubSubSerializedMessage>,
    timeout_handler: Option<SpawnHandle>,
    target: Recipient<PubSubMessageBatch>,
    target_topic: &'static str,
    on_stop: Option<Sender<()>>,
}

impl MessageBatchingActor {
    fn send_batch(&mut self, ctx: &mut Context<MessageBatchingActor>) {
        let batch = self.buffer.drain(..).collect();
        self.timeout_handler = None;

        self.target
            .borrow()
            .do_send(PubSubMessageBatch {
                messages: batch,
                target_topic: self.target_topic,
            });
    }
}
impl Actor for MessageBatchingActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(BATCH_SIZE * 5);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        println!("BatchActor stopping");
        if self.buffer.len() > 0 {
            println!("==>>>>> LAST BATCH");
            self.send_batch(ctx);
        }
        Running::Stop
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        println!("BatchActor stopped");
        let sender = mem::replace(&mut self.on_stop, None);
        match sender {
            None => {}
            Some(s) => { s.send(()); }
        }
    }
}

impl Handler<PubSubSerializedMessage> for MessageBatchingActor {
    type Result = ();
    fn handle(&mut self, msg: PubSubSerializedMessage, ctx: &mut Self::Context) -> Self::Result {
        self.buffer.push(msg);

        match self.timeout_handler {
            Some(_) => (),
            None => {
                self.timeout_handler = Some(ctx.run_later(BATCH_TIMEOUT, |act, ctx| {
                    act.send_batch(ctx);
                }))
            }
        };

        if self.buffer.len() >= BATCH_SIZE {
            match self.timeout_handler {
                None => true,
                Some(handler) => ctx.cancel_future(handler),
            };
            self.send_batch(ctx);
        }
    }
}

impl Handler<ShutDown> for MessageBatchingActor {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: ShutDown, ctx: &mut Self::Context) -> Self::Result {
        let (tx, tr) = tokio::sync::oneshot::channel();
        self.on_stop = Some(tx);
        ctx.stop();
        Box::pin(async move {
            println!("MessagePushActor waiting to be stopped");
            tr.await;
            println!("MessagePushActor is stopped");
        })
    }
}

// ###################################### to file

#[derive(Message)]
#[rtype(result = "()")]
struct PubSubMessageBatch {
    messages: Vec<PubSubSerializedMessage>,
    target_topic: &'static str,
}

struct MessagePushActor {
    cnt: usize,
    on_stop: Option<Sender<()>>,
}

impl Actor for MessagePushActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(10);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        println!("MessagePushActor stopping");
        Running::Stop
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        let sender = mem::replace(&mut self.on_stop, None);
        match sender {
            None => {}
            Some(s) => { s.send(()); }
        }
        println!("MessagePushActor stopped");
    }
}

impl Handler<PubSubMessageBatch> for MessagePushActor {
    type Result = ();
    fn handle(&mut self, msg: PubSubMessageBatch, ctx: &mut Self::Context) -> Self::Result {
        self.cnt += msg.messages.len();
        let new_cnt = self.cnt;

        println!(
            "Pushing batch to pubsub topic {}.Total messages send {}!",
            msg.target_topic, new_cnt
        );

        tokio::time::sleep(Duration::from_secs(5))
            .into_actor(self)
            .map(|res, _act, ctx| println!("Batch is pushed!"))
            .wait(ctx);
    }
}

impl Handler<ShutDown> for MessagePushActor {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: ShutDown, ctx: &mut Self::Context) -> Self::Result {
        let (tx, tr) = tokio::sync::oneshot::channel();
        self.on_stop = Some(tx);
        ctx.stop();
        Box::pin(async move {
            println!("MessagePushActor waiting to be stopped");
            tr.await;
            println!("MessagePushActor is stopped");
        })
    }
}

// ###################################### to file

struct PubSub {
    batch_actors: HashMap<&'static str, Addr<MessageBatchingActor>>,
    push_actor: Addr<MessagePushActor>,
}

struct MessageSender<T: 'static + Unpin + Send> {
    target: Recipient<PubSubSerializedMessage>,
    resource_type: PhantomData<T>,
}

impl<T: 'static + Unpin + Send> MessageSender<T> {
    async fn send(self: &MessageSender<T>, message: T) -> Result<(), &'static str> {
        let serialized: Vec<u8> = Vec::new(); // message.to_byte_array
        let res = self.target.send(PubSubSerializedMessage(serialized)).await;
        match res {
            Ok(_) => Ok(()),
            _ => Err("Receiver is closed"),
        }
    }
}

impl PubSub {
    fn create() -> PubSub {
        PubSub {
            batch_actors: Default::default(),
            push_actor: MessagePushActor { cnt: 0, on_stop: None }.start(),
        }
    }

    fn create_sender<T: 'static + Unpin + Send>(
        self: &mut PubSub,
        target_topic: &'static str,
    ) -> MessageSender<T> {
        let batch_recipient = match self.batch_actors.entry(target_topic) {
            Entry::Occupied(mut o) => o.get_mut().clone().recipient(),
            Entry::Vacant(v) => {
                let push_recipient = self.push_actor.clone().recipient();

                let actor = MessageBatchingActor {
                    buffer: vec![],
                    timeout_handler: None,
                    target: push_recipient,
                    target_topic,
                    on_stop: None
                }
                .start();
                let recipient = actor.clone().recipient();
                v.insert(actor);
                recipient
            }
        };

        MessageSender {
            target: batch_recipient,
            resource_type: PhantomData::default()
        }
    }

    async fn run_until(
        self: &mut PubSub,
    ) {
        match tokio::signal::ctrl_c().await {
            Ok(_) => {}
            Err(_) => {}
        }
        for x in self.batch_actors.values().into_iter() {
            println!("Waiting to shutdown batch actor");
            let r = x.send(ShutDown).await;
            match r {
                Ok(res) => res,
                Err(_) => println!("Failed to shutdown batch actor")
            }
        }
        self.push_actor.send(ShutDown).await;
    }
}

// ######################################

struct PubSubEventA(i32);
struct PubSubEventB;
struct PubSubEventC;

#[actix::main]
async fn main() {
    let mut pub_sub = PubSub::create();

    let sender_a: MessageSender<PubSubEventA> = pub_sub.create_sender("topic1");

    let t = tokio::task::spawn(async move {
        for i in 1..10000 {
            match sender_a.borrow().send(PubSubEventA(i)).await {
                Ok(_) => println!("Send {} message!", i),
                Err(_) => {
                    println!("Sender is closed!");
                    break;
                }
            };

            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    });

    println!("Waiting for Ctrl-C...");
    pub_sub.run_until().await;
    println!("Gracefully stopped");
    t.await;
}
