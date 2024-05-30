use crate::server::*;
use std::time::Instant;

/// The Subscriber can be used to listen to messages from specific probes
pub trait Subscriber {
    /// called when a new cycle's messages arrived,
    /// takes the Subscribe probes's message sent in the cycle
    /// the return value will be put into the server
    fn update(&mut self, messages: Vec<B2RMessage>) -> Vec<R2BMessage>;
    /// Specify the ID of the subscribed probe.
    fn subscribed_ids(&self) -> Vec<u32>;
}

/// A wrapper for B2RServer that to used the server and getter in event driven style
pub struct B2RPublisher {
    server: B2RServer,
    cycle_getter: CycleGetter,
    subscribers: Vec<Box<dyn Subscriber>>,
}

impl B2RPublisher {
    /// Creates a new B2RPublisher instance with the specified socket path.
    pub fn new_with(path: &str) -> Self {
        let server = B2RServer::new_with(path);
        let cycle_getter = CycleGetter::new(&server);
        B2RPublisher {
            server,
            cycle_getter,
            subscribers: Vec::new(),
        }
    }

    /// Adds a subscriber to the publisher.
    pub fn add_subscriber(&mut self, subscriber: impl Subscriber + 'static) {
        self.subscribers.push(Box::new(subscriber));
    }

    /// Serves the server and starts processing messages.
    pub fn serve(&mut self) {
        let handle = self.server.serve();
        let mut cycle: u32 = 0;
        loop {
            let mut messages: Vec<B2RMessage> = Vec::new();

            // get all messages in a cycle
            let start_time = Instant::now();
            loop {
                let server_cycle = self.server.current_cycle();
                if server_cycle > cycle || start_time.elapsed().as_secs() >= 1 {
                    cycle = server_cycle;
                    break;
                }
                let cycle_message = self.cycle_getter.get_cycle_message();
                // need to get again if the current cycle messages didn't end
                messages.extend(cycle_message);
            }

            // call update for the subscribers
            for subscriber in &mut self.subscribers {
                let subscribed_ids = subscriber.subscribed_ids();

                let subscribed_messages = messages
                    .iter()
                    .filter(|message| subscribed_ids.contains(&message.id))
                    .cloned()
                    .collect();

                let put_messages = subscriber.update(subscribed_messages);

                for put_message in put_messages {
                    self.server.put(put_message.id, put_message.message);
                }
            }

            //stopped when there are no more message
            if !self.server.running() {
                break;
            }
        }

        let _ = handle.join();
    }
}
