#![feature(variant_count)]

// TODO: Add docstrings to explain the concepts laid out here

use core::time::Duration;

use polite_agent::{PoliteAgent, PoliteAgentConfig, PoliteAgentMessage};
use post_haste::init_postmaster;
use tokio::time::sleep;

enum Payloads {
    PoliteMessage(PoliteAgentMessage),
}

#[derive(Debug, Clone, Copy)]
enum Address {
    AgentA,
    AgentB,
    AgentC,
}

init_postmaster!(Address, Payloads);

#[tokio::main]
async fn main() {
    register_agent!(
        AgentA,
        PoliteAgent,
        PoliteAgentConfig {
            custom_greeting: None,
            reply_delay: Duration::from_secs(1)
        }
    );
    register_agent!(
        AgentB,
        PoliteAgent,
        PoliteAgentConfig {
            custom_greeting: Some("Good day!".to_string()),
            reply_delay: Duration::from_secs(2)
        }
    );
    register_agent!(
        AgentC,
        PoliteAgent,
        PoliteAgentConfig {
            custom_greeting: Some("Ahoy!".to_string()),
            reply_delay: Duration::from_secs(3)
        }
    );

    postmaster::send(
        Address::AgentA,
        Address::AgentB,
        Payloads::PoliteMessage(PoliteAgentMessage::Hello),
    )
    .await
    .unwrap();
    postmaster::send(
        Address::AgentB,
        Address::AgentC,
        Payloads::PoliteMessage(PoliteAgentMessage::Hello),
    )
    .await
    .unwrap();
    postmaster::send(
        Address::AgentC,
        Address::AgentA,
        Payloads::PoliteMessage(PoliteAgentMessage::Hello),
    )
    .await
    .unwrap();

    loop {
        sleep(Duration::from_secs(60)).await;
        let diagnostics = postmaster::get_diagnostics();
        println!("Postmaster diagnostics:");
        println!("Messages sent: {}", diagnostics.messages_sent);
        println!("Send failures: {}", diagnostics.send_failures);
    }
}

mod polite_agent {
    use crate::{Address, Payloads, postmaster};
    use core::time::Duration;
    use post_haste::agent::Agent;

    pub(super) struct PoliteAgent {
        address: Address,
        greeting: Option<String>,
        reply_delay: Duration,
    }

    pub(super) struct PoliteAgentConfig {
        pub custom_greeting: Option<String>,
        pub reply_delay: Duration,
    }

    pub(super) enum PoliteAgentMessage {
        Hello,
        Greeting(String),
    #[allow(private_interfaces)]
        Internal(InternalMessage),
    }

    enum InternalMessage {
        TimerExpired { hello_source: Address },
    }

    impl Agent for PoliteAgent {
        type Address = Address;

        type Message = postmaster::Message;

        type Config = PoliteAgentConfig;

        async fn create(address: Self::Address, config: Self::Config) -> Self {
            Self {
                address,
                greeting: config.custom_greeting,
                reply_delay: config.reply_delay,
            }
        }

        async fn run(mut self, mut inbox: post_haste::agent::Inbox<Self::Message>) -> ! {
            loop {
                let received_message = inbox.recv().await.unwrap();
                match received_message.payload {
                    crate::Payloads::PoliteMessage(PoliteAgentMessage::Hello) => {
                        self.handle_hello(received_message.source).await
                    }
                    crate::Payloads::PoliteMessage(PoliteAgentMessage::Greeting(greeting)) => {
                        self.handle_greeting(received_message.source, &greeting)
                            .await
                    }
                    crate::Payloads::PoliteMessage(PoliteAgentMessage::Internal(
                        InternalMessage::TimerExpired { hello_source },
                    )) => self.send_reply(hello_source).await,
                }
            }
        }
    }

    impl PoliteAgent {
        async fn handle_hello(&mut self, source: Address) {
            println!("{:?} got hello from {source:?}", self.address);
            self.start_timer(source).await
        }

        async fn handle_greeting(&mut self, source: Address, greeting: &str) {
            println!(
                "{:?} got greeting from {source:?}: {greeting}",
                self.address
            );
            self.start_timer(source).await
        }

        async fn send_reply(&mut self, destination: Address) {
            let payload = if let Some(greeting) = &self.greeting {
                Payloads::PoliteMessage(PoliteAgentMessage::Greeting(greeting.to_string()))
            } else {
                Payloads::PoliteMessage(PoliteAgentMessage::Hello)
            };

            postmaster::send(destination, self.address, payload)
                .await
                .unwrap();
        }

        async fn start_timer(&mut self, hello_source: Address) {
            postmaster::message(
                self.address,
                self.address,
                Payloads::PoliteMessage(PoliteAgentMessage::Internal(
                    InternalMessage::TimerExpired { hello_source },
                )),
            )
            .with_delay(self.reply_delay)
            .send()
            .await
            .unwrap();
        }
    }
}
