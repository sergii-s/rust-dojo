use actix::prelude::*;
use std::borrow::Borrow;
use std::time::Duration;
use std::collections::HashMap;
use std::ops::Add;
use std::marker::PhantomData;
use std::collections::hash_map::Entry;

// ###################################### to file

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
    type Result = ();

    fn handle(&mut self, msg: PubSubMessage<T>, ctx: &mut Self::Context) {
        println!("Got message!");
        //todo serialize using protobuf T to u8[]
        let serialized : Vec<u8> = Vec::new();
        self.target.borrow().do_send(PubSubSerializedMessage(serialized));
    }
}


// ###################################### to file


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
    fn batch(&mut self) {
        let batch = self.buffer.drain(..).collect();
        self.target.borrow().do_send(PubSubMessageBatch{messages : batch, target : self.target_topic});
        self.timeout_handler = None;
    }
}
impl Actor for MessageBatchingActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(3);
    }
}

impl Handler<PubSubSerializedMessage> for MessageBatchingActor {
    type Result = ();
    fn handle(&mut self, msg: PubSubSerializedMessage, ctx: &mut Self::Context) {
        self.buffer.push(msg);

        match self.timeout_handler {
            Some(_) => (),
            None => {
                self.timeout_handler = Some(ctx.run_later(BATCH_TIMEOUT, |act, ctx| {
                    act.batch();
                }))
            }
        };

        if self.buffer.len() >= BATCH_SIZE {
            match self.timeout_handler {
                None => true,
                Some(handler) => ctx.cancel_future(handler),
            };
            self.batch();
        }
    }
}


// ###################################### to file

#[derive(Message)]
#[rtype(result = "()")]
struct PubSubMessageBatch {
    messages : Vec <PubSubSerializedMessage>,
    target : &'static str
}

struct MessagePushActor;

impl Actor for MessagePushActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(3);
    }
}

impl Handler<PubSubMessageBatch> for MessagePushActor {
    type Result = ();
    fn handle(&mut self, msg: PubSubMessageBatch, ctx: &mut Self::Context) {
        println!("Pushing {} batch to pubsub topic {}!", msg.messages.len(), msg.target);
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
            push_actor: MessagePushActor.start()
        }
    }
    fn create_sender<T: 'static + Unpin + Send>(self: &mut PubSub, target_topic: &'static str) -> Addr<MessageSenderActor<T>> {
        //todo do not clone
        let rec = self.push_actor.clone().recipient();
        let batch_actor : &Addr<MessageBatchingActor> =
            self.batch_actors.entry(target_topic).or_insert_with( ||  {
                    MessageBatchingActor {
                        buffer: vec![],
                        timeout_handler: None,
                        target: rec,
                        target_topic
                    }.start()
                });

        let sender_actor = MessageSenderActor {
            target: batch_actor.borrow().recipient(),
            resource_type: PhantomData
        };
        sender_actor.start()
    }
}

// ######################################

struct PubSubEventA;
struct PubSubEventB;

#[actix::main]
async fn main() {
    let mut pub_sub = PubSub::create();

    let senderA : Addr<MessageSenderActor<PubSubEventA>> = pub_sub.create_sender("topic1");
    let senderB : Addr<MessageSenderActor<PubSubEventB>> = pub_sub.create_sender("topic1");

    let _ = senderA.borrow().do_send(PubSubMessage(Box::new(PubSubEventA)));
    let _ = senderA.borrow().do_send(PubSubMessage(Box::new(PubSubEventA)));
    // let _ = sender.do_send(PubSubMessage);
    // let _ = sender.do_send(PubSubMessage);
    // let _ = sender.do_send(PubSubMessage);
    // let _ = sender.do_send(PubSubMessage);
    // let _ = sender.do_send(PubSubMessage);
    // let _ = sender.do_send(PubSubMessage);

    tokio::time::sleep(Duration::from_secs(5)).await;
}
