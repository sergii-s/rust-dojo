use actix::prelude::*;
use std::borrow::Borrow;
use std::time::Duration;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::collections::hash_map::Entry;

// ###################################### to file - MessageSenderActor

#[derive(Message)]
#[rtype(result = "()")]
struct PubSubMessage<T: Send>(Box<T>);

/// Actor that provides order shipped event subscriptions
struct MessageSenderActor<T: Send> {
    target: Recipient<PubSubSerializedMessage>,
    resource_type: PhantomData<T>,
}

impl<T: 'static + Unpin + Send> Actor for MessageSenderActor<T> {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(10);
    }
}

impl<T: 'static + Unpin + Send> Handler<PubSubMessage<T>> for MessageSenderActor<T> {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: PubSubMessage<T>, ctx: &mut Self::Context) -> Self::Result {
        println!("Got message!");
        //todo serialize using protobuf T to u8[]
        let serialized : Vec<u8> = Vec::new();
        let target = self.target.clone();
        Box::pin(async move {
            target.send(PubSubSerializedMessage(serialized)).await;
        })
    }
}


// ###################################### to file - MessageBatchingActor


const BATCH_SIZE: usize = 3;
const BATCH_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Message)]
#[rtype(result = "()")]
struct PubSubSerializedMessage(Vec<u8>);

struct MessageBatchingActor {
    buffer: Vec<PubSubSerializedMessage>,
    timeout_handler: Option<SpawnHandle>,
    target: Recipient<PubSubMessageBatch>,
    target_topic: &'static str,
}

impl MessageBatchingActor {
    fn drain_batch(&mut self) -> Vec<PubSubSerializedMessage> {
        let batch = self.buffer.drain(..).collect();
        self.timeout_handler = None;
        batch
    }
}
impl Actor for MessageBatchingActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(BATCH_SIZE*2);
    }
}

impl Handler<PubSubSerializedMessage> for MessageBatchingActor {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: PubSubSerializedMessage, ctx: &mut Self::Context) -> Self::Result {
        self.buffer.push(msg);

        match self.timeout_handler {
            Some(_) => (),
            None => {
                self.timeout_handler = Some(ctx.run_later(BATCH_TIMEOUT, |act, ctx| {
                    let batch = act.drain_batch();
                    let target = act.target.borrow();
                    let target_topic = act.target_topic;
                    target.do_send(PubSubMessageBatch{messages : batch, target_topic });
                }))
            }
        };

        if self.buffer.len() >= BATCH_SIZE {
            match self.timeout_handler {
                None => true,
                Some(handler) => ctx.cancel_future(handler),
            };
            let batch = self.drain_batch();
            let target = self.target.clone();
            let target_topic = self.target_topic;

            Box::pin(async move {
                target.borrow().send(PubSubMessageBatch{messages : batch, target_topic }).await;
            })
        } else {
            Box::pin(async move {})
        }
    }
}


// ###################################### to file

#[derive(Message)]
#[rtype(result = "()")]
struct PubSubMessageBatch {
    messages : Vec <PubSubSerializedMessage>,
    target_topic : &'static str
}

struct MessagePushActor {
    cnt: i32
}

impl Actor for MessagePushActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(10);
    }
}

impl Handler<PubSubMessageBatch> for MessagePushActor {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: PubSubMessageBatch, ctx: &mut Self::Context) -> Self::Result {
        self.cnt += 1;
        let new_cnt = self.cnt;
        Box::pin(async move {
            println!("Pushing {} batch to pubsub topic {}!", new_cnt, msg.target_topic);
            tokio::time::sleep(Duration::from_secs(5)).await;
            println!("Pushing {} batch to pubsub topic {} DONE!", new_cnt, msg.target_topic);
        })
    }
}

// ###################################### to file

struct PubSub {
    batch_actors : HashMap<&'static str, Addr<MessageBatchingActor>>,
    push_actor : Addr<MessagePushActor>,
}

impl PubSub {
    fn create() -> PubSub {
        PubSub {
            batch_actors: Default::default(),
            push_actor: MessagePushActor{cnt : 0}.start()
        }
    }
    fn create_sender<T: 'static + Unpin + Send>(self: &mut PubSub, target_topic: &'static str) -> Addr<MessageSenderActor<T>> {
        let batch_recipient  =
            match self.batch_actors.entry(target_topic) {
                Entry::Occupied(mut o) => o.get_mut().clone().recipient(),
                Entry::Vacant(v) => {
                    let push_recipient = self.push_actor.clone().recipient();
                    let actor = MessageBatchingActor {
                        buffer: vec![],
                        timeout_handler: None,
                        target: push_recipient,
                        target_topic
                    }.start();
                    let recipient = actor.clone().recipient();
                    v.insert(actor);
                    recipient
                }
            };

        let sender_actor = MessageSenderActor {
            target: batch_recipient,
            resource_type: PhantomData
        };
        sender_actor.start()
    }
}

// ######################################

struct PubSubEventA;
struct PubSubEventB;
struct PubSubEventC;

#[actix::main]
async fn main() {
    let mut pub_sub = PubSub::create();

    let sender_a: Addr<MessageSenderActor<PubSubEventA>> = pub_sub.create_sender("topic1");
    let sender_b: Addr<MessageSenderActor<PubSubEventB>> = pub_sub.create_sender("topic1");
    let sender_c: Addr<MessageSenderActor<PubSubEventC>> = pub_sub.create_sender("topic2");

    for i in 1..10000 {
        let _ = sender_a.borrow().send(PubSubMessage(Box::new(PubSubEventA))).await;
        let _ = sender_a.borrow().send(PubSubMessage(Box::new(PubSubEventA))).await;
        let _ = sender_a.borrow().send(PubSubMessage(Box::new(PubSubEventA))).await;
        // let _ = sender_a.borrow().do_send(PubSubMessage(Box::new(PubSubEventA)));
        // let _ = sender_a.borrow().do_send(PubSubMessage(Box::new(PubSubEventA)));
        // let _ = sender_a.borrow().do_send(PubSubMessage(Box::new(PubSubEventA)));
        // let _ = sender_b.borrow().do_send(PubSubMessage(Box::new(PubSubEventB)));
        // let _ = sender_b.borrow().do_send(PubSubMessage(Box::new(PubSubEventB)));
        // let _ = sender_c.borrow().do_send(PubSubMessage(Box::new(PubSubEventC)));
        // let _ = sender_c.borrow().do_send(PubSubMessage(Box::new(PubSubEventC)));
        // let _ = sender_c.borrow().do_send(PubSubMessage(Box::new(PubSubEventC)));
        // let _ = sender_c.borrow().do_send(PubSubMessage(Box::new(PubSubEventC)));
        // tokio::time::sleep(Duration::from_millis(200)).await;
    }


    tokio::time::sleep(Duration::from_secs(100)).await;
}
