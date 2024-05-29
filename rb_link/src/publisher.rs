use crate::server::*;
use crate::SHUT_DOWN_ID;

pub trait Subscribe {
    fn update(&mut self, messages: Vec<B2RMessage>) -> Vec<R2BMessage>;
    fn subscribed_ids(&self) -> Vec<u32>;
}

pub struct B2RPublisher {
    server: B2RServer,
    cycle_getter: CycleGetter,
    subscribers: Vec<Box<dyn Subscribe>>,
}

impl B2RPublisher {
    pub fn new_with(path: &str) -> Self {
        let server = B2RServer::new_with(path);
        let cycle_getter = CycleGetter::new(&server);
        B2RPublisher {
            server,
            cycle_getter,
            subscribers: Vec::new(),
        }
    }

    pub fn serve(&mut self) {
        let handle = self.server.serve();
        let mut cycle: u32 = 0;
        loop {
            let mut messages: Vec<B2RMessage> = Vec::new();

            loop {
                let cycle_message = self.cycle_getter.get_cycle_message();
                if cycle_message.is_empty() || cycle_message[0].cycles > cycle {
                    cycle = cycle_message[0].cycles;
                    break;
                }
                messages.extend(cycle_message);
            }

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

            if messages.iter().any(|message| message.id == SHUT_DOWN_ID) {
                break;
            }
        }

        let _ = handle.join();
    }
}
