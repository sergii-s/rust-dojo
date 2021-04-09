use actix::prelude::*;
use std::borrow::Borrow;
use std::time::Duration;

struct PubSubEventA;
struct PubSubEventB;

#[derive(Message)]
#[rtype(result = "()")]
struct PubSubMessage;

#[derive(Message)]
#[rtype(result = "()")]
struct PubSubMessageBatch(Vec<PubSubMessage>);

/// Actor that provides order shipped event subscriptions
struct MessageSender {
    buffer : Vec<PubSubMessage>,
    target : Recipient<PubSubMessageBatch>
}

impl MessageSender {
    fn batch(&mut self) {
        let batch = self.buffer.drain(..).collect();
        self.target.borrow().do_send(PubSubMessageBatch(batch));
    }
}

impl Actor for MessageSender {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(10);
    }
}

impl Handler<PubSubMessage> for MessageSender {
    type Result = ();

    fn handle(&mut self, msg: PubSubMessage, ctx: &mut Self::Context) {
        println!("Got message!");
        self.buffer.push(msg);
        if self.buffer.len() >= 3 {
            self.batch();
        }
    }
}

struct BatchMessageSenderActor;
impl Actor for BatchMessageSenderActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(1);
    }
}
impl Handler<PubSubMessageBatch> for BatchMessageSenderActor {
    type Result = ();
    fn handle(&mut self, msg: PubSubMessageBatch, ctx: &mut Self::Context) {
        println!("Got {} batch!", msg.0.len());
    }
}

#[actix::main]
async fn main() {
    let batch_agent = BatchMessageSenderActor.start();
    let sender = MessageSender {
        buffer : Vec::new(),
        target : batch_agent.recipient()
    }.start();
    println!("started!");

    let _ = sender.do_send(PubSubMessage);
    let _ = sender.do_send(PubSubMessage);
    let _ = sender.do_send(PubSubMessage);
    let _ = sender.do_send(PubSubMessage);
    let _ = sender.do_send(PubSubMessage);
    let _ = sender.do_send(PubSubMessage);
    let _ = sender.do_send(PubSubMessage);

    tokio::time::sleep(Duration::from_secs(1)).await;
}

