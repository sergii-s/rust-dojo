use actix::prelude::*;
use std::borrow::Borrow;
use std::time::Duration;

const BATCH_SIZE: usize = 3;
const BATCH_TIMEOUT: Duration = Duration::from_secs(1);


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
    buffer: Vec<PubSubMessage>,
    timeout_handler: Option<SpawnHandle>,
    target: Recipient<PubSubMessageBatch>,
}

impl MessageSender {
    fn batch(&mut self) {
        let batch = self.buffer.drain(..).collect();
        self.target.borrow().do_send(PubSubMessageBatch(batch));
        self.timeout_handler = None;
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

struct BatchMessageSenderActor;
impl Actor for BatchMessageSenderActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(3);
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
        buffer: Vec::new(),
        target: batch_agent.recipient(),
        timeout_handler: None,
    }
    .start();
    println!("started!");

    let _ = sender.do_send(PubSubMessage);
    let _ = sender.do_send(PubSubMessage);
    let _ = sender.do_send(PubSubMessage);
    let _ = sender.do_send(PubSubMessage);
    let _ = sender.do_send(PubSubMessage);
    let _ = sender.do_send(PubSubMessage);
    let _ = sender.do_send(PubSubMessage);

    tokio::time::sleep(Duration::from_secs(5)).await;
}
