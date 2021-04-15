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


// ###################################### to file - BATCHING ACTOR


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
        ctx.set_mailbox_capacity(BATCH_SIZE*2);
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

    let _ = sender_a.borrow().do_send(PubSubMessage(Box::new(PubSubEventA)));
    let _ = sender_a.borrow().do_send(PubSubMessage(Box::new(PubSubEventA)));
    let _ = sender_b.borrow().do_send(PubSubMessage(Box::new(PubSubEventB)));
    let _ = sender_b.borrow().do_send(PubSubMessage(Box::new(PubSubEventB)));
    let _ = sender_c.borrow().do_send(PubSubMessage(Box::new(PubSubEventC)));
    let _ = sender_c.borrow().do_send(PubSubMessage(Box::new(PubSubEventC)));
    let _ = sender_c.borrow().do_send(PubSubMessage(Box::new(PubSubEventC)));
    let _ = sender_c.borrow().do_send(PubSubMessage(Box::new(PubSubEventC)));
    // let _ = sender.do_send(PubSubMessage);
    // let _ = sender.do_send(PubSubMessage);
    // let _ = sender.do_send(PubSubMessage);
    // let _ = sender.do_send(PubSubMessage);
    // let _ = sender.do_send(PubSubMessage);
    // let _ = sender.do_send(PubSubMessage);

    tokio::time::sleep(Duration::from_secs(5)).await;
}
