//! The purpose of this example is to demonstrate some useful patterns and concepts which take advantage of the post-haste library to great effect.
//! The logic of this example is very similar to that of the [tokio_basic](tokio_basic.rs)example: simple Agents are created which respond to a "hello" message in kind.
//! "Hello" messages are then sent from the main task to the Agents, with the source address given as one of the other Agents.
//! This will prompt the Agent to respond with its own "hello" back to the source, initiating an infinite loop.

#![feature(variant_count)]
use core::time::Duration;

use polite_agent::{PoliteAgent, PoliteAgentConfig, PoliteAgentMessage};
use post_haste::init_postmaster;
use tokio::time::sleep;

/// This enum describes the messages used by the system.
/// This could be arranged as a single enum of all possible messages, however I prefer to group messages by the actor they are associated with.
/// In this case, as there is only one actor there is only one Payload variant.
enum Payloads {
    /// This variant covers all messages associated with the Polite Agent.
    PoliteMessage(PoliteAgentMessage),
}

/// This enum provides all Agent addresses.
/// Each Agent must be assigned a unique address upon registration with the Postmaster.
/// As indicated by the signature of the Agent trait's `run()` method, Agents are expected to live for the lifetime of the application. This ensures that addresses are always valid and messages aren't accidentally sent to an unoccupied address.
#[derive(Debug, Clone, Copy)]
enum Address {
    AgentA,
    AgentB,
    AgentC,
}

/// This macro call generates all the functionality associated with the Postmaster.
/// The Address and Payload enum types are provided as arguments, as they are required for the functionality of the Postmaster.
init_postmaster!(Address, Payloads);

/// This module provides all functionality and types associated with the Polite Agent.
/// Usually this would be in its own file, however here it is presented as a module so that the example is a single, self-reliant file.
mod polite_agent {
    use crate::{Address, Payloads, postmaster};
    use core::time::Duration;
    use post_haste::agent::Agent;

    /// This struct acts as the "instance" of the Agent, and is ususally used to hold state and configuration information
    pub(super) struct PoliteAgent {
        /// Storing the address of the agent as a member field is optional, however it means that `self.address` can be used as the source when sending messages.
        /// In cases with multiple instances of an Agent type are running concurrently, this becomes very necessary.
        address: Address,
        /// Our PoliteAgent can be configured with an optional custom greeting, rather than having to rely on the standard "hello".
        greeting: Option<String>,
        /// In order to avoid spamming stdout with messages, we also configure the PoliteAgent with a delay.
        /// When a message is received, it will wait for this amount of time before sending its reply.
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

