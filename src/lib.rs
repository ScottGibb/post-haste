#![no_std]

pub mod agent;
pub mod error;

pub mod dependencies {
    pub use const_env::from_env;
    pub use embassy_executor::{task, SpawnToken, Spawner};
    pub use embassy_sync::{
        blocking_mutex::raw::NoopRawMutex,
        channel::{Channel, DynamicSender},
        mutex::Mutex,
    };
    pub use embassy_time::{Duration, Timer, WithTimeout};
    pub use portable_atomic::{AtomicU32, AtomicUsize};
}
pub use error::PostmasterError;

#[macro_export]
macro_rules! init_postmaster {
    ($address_enum:ty, $payload_enum:ty) => {
        mod postmaster {
            use super::{$address_enum, $payload_enum};
            use post_haste::PostmasterError;
            use post_haste::dependencies::*;

            const ADDRESS_COUNT: usize = core::mem::variant_count::<$address_enum>();

            #[macro_export]
            macro_rules! register_agent {
                ($spawner:ident, $agent_address:ident, $agent:ty, $config:expr, $queue_size: expr) => {{
                    use post_haste::dependencies::{NoopRawMutex, Channel, task};
                    use post_haste::agent::Agent;
                    use crate::postmaster::Message;
                    struct Mailbox {
                        pub inner: Channel<NoopRawMutex, Message, $queue_size>
                    }

                    unsafe impl Sync for Mailbox{}
                    static MAILBOX: Mailbox = Mailbox{ inner: Channel::new()};

                    let agent = <$agent>::create(<$address_enum>::$agent_address, $config).await;
                    postmaster::register(<$address_enum>::$agent_address, MAILBOX.inner.sender().into()).await.inspect(|_| {

                        #[task]
                        async fn run_agent(agent: $agent) {
                            agent.run(MAILBOX.inner.receiver().into()).await
                        }
                        $spawner.must_spawn(run_agent(agent));
                    })
                }};
                ($spawner:ident, $agent_address:ident, $agent:ty, $config:expr) => {
                    register_agent!($spawner, $agent_address, $agent, $config, 1)
                }
            }

            pub async fn register(
                address: $address_enum,
                mailbox: DynamicSender<'static, Message>,
            ) -> Result<(), PostmasterError> {
                postmaster_internal::register(address, mailbox).await
            }

            pub async fn send(destination: $address_enum, source: $address_enum, payload: $payload_enum) -> Result<(), PostmasterError> {
                postmaster_internal::send_internal(destination, Message{ source, payload }, None).await
            }

            pub fn try_send(destination: $address_enum, source: $address_enum, payload: $payload_enum) -> Result<(), PostmasterError> {
                postmaster_internal::try_send_internal(destination, Message{ source, payload })
            }

            pub fn message(destination: $address_enum, source: $address_enum, payload: $payload_enum) -> MessageBuilder {
                MessageBuilder { destination, message: Message{ source, payload }, timeout: None, delay: None }
            }

            impl MessageBuilder {
                pub fn with_timeout(mut self, timeout: Duration) -> Self {
                    self.timeout.replace(timeout);
                    self
                }

                pub fn with_delay(mut self, delay: Duration) -> Self {
                    self.delay.replace(delay);
                    self
                }

                pub async fn send(self) -> Result<(), PostmasterError> {
                    match self.delay {
                        Some(delay) => postmaster_internal::spawn_delayed_send(self.destination, self.message, delay, self.timeout).await,
                        None => postmaster_internal::send_internal(self.destination, self.message, self.timeout).await
                    }
                }
            }

            pub struct Message {
                pub source: $address_enum,
                pub payload: $payload_enum,
            }

            pub struct MessageBuilder {
                destination: $address_enum,
                message: Message,
                timeout: Option<Duration>,
                delay: Option<Duration>,
            }

            mod postmaster_internal {
                use super::{ADDRESS_COUNT, $address_enum, Message, PostmasterError, $payload_enum};
                use core::cell::RefCell;
                use core::sync::atomic::{Ordering};
                use post_haste::dependencies::*;

                #[post_haste::dependencies::from_env]
                const DELAYED_MESSAGE_POOL_SIZE: usize = 8;

                pub(super) async fn register(address: $address_enum, mailbox: DynamicSender<'static, Message>) -> Result<(), PostmasterError> {
                    let mut senders = POSTMASTER.senders.lock().await;
                    if senders[address as usize].is_none() {
                        senders[address as usize].replace(mailbox);
                        Ok(())
                    } else {
                        return Err(PostmasterError::AddressAlreadyTaken);
                    }

                }

                pub(super) async fn send_internal(
                    destination: $address_enum,
                    message: Message,
                    timeout: Option<Duration>,
                ) -> Result<(), PostmasterError> {
                    let timeout = match timeout {
                        Some(duration) => duration,
                        None => Duration::from_micros(
                            POSTMASTER.timeout_us.load(Ordering::Relaxed).into(),
                        ),
                    };
                    evaluate_diagnostics(
                        async {
                            match POSTMASTER.senders.lock().await[destination as usize] {
                                None => Err(PostmasterError::NoRecipient),
                                Some(sender) => {
                                    sender.send(message).await;
                                    Ok(())
                                }
                            }
                        }
                        .with_timeout(timeout)
                        .await?,
                    )
                }

                pub(super) fn try_send_internal(destination: $address_enum, message: Message) -> Result<(), PostmasterError> {
                    evaluate_diagnostics(
                        match POSTMASTER.senders.try_lock()?[destination as usize] {
                            None => Err(PostmasterError::NoRecipient),
                            Some(sender) => {
                                sender.try_send(message)?;
                                Ok(())
                            }
                        })
                }

                pub(super) async fn spawn_delayed_send(destination: $address_enum, message: Message, delay: Duration, timeout: Option<Duration>)  -> Result<(), PostmasterError> {
                    if let Some(spawner) = *POSTMASTER.spawner.borrow(){
                            Ok(spawner.spawn(delayed_send(destination, message, delay, timeout))?)
                        } else {
                            send_internal(destination, message, timeout).await
                        }
                }

                #[task(pool_size=DELAYED_MESSAGE_POOL_SIZE)]
                pub(super) async fn delayed_send(destination: $address_enum, message: Message, delay: Duration, timeout: Option<Duration>) {
                    Timer::after(delay).await;
                    let source = message.source;
                    match send_internal(destination, message, timeout).await {
                        Ok(_) => (),
                        Err(error) => (), // TODO: Can we find a way to convey back to the source that the sending failed?
                    }
                }

                #[task(pool_size=DELAYED_MESSAGE_POOL_SIZE)]
                pub(super) async fn delayed_try_send(destination: $address_enum, message: Message, delay: Duration) {
                    Timer::after(delay).await;
                    match try_send_internal(destination, message) {
                        Ok(_) => (),
                        Err(error) => (), // TODO: Can we find a way to convey back to the source that the sending failed?
                    }
                }

                struct Postmaster<'a> {
                    senders:
                        Mutex<NoopRawMutex, [Option<DynamicSender<'a, Message>>; ADDRESS_COUNT]>,
                    timeout_us: AtomicU32,
                    spawner: RefCell<Option<Spawner>>,
                    messages_sent: AtomicUsize,
                    send_failures: AtomicUsize,
                }

                unsafe impl Sync for Postmaster<'_> {}

                static POSTMASTER: Postmaster = Postmaster {
                    senders: Mutex::new([None; ADDRESS_COUNT]),
                    timeout_us: AtomicU32::new(100),
                    spawner: RefCell::new(None),
                    messages_sent: AtomicUsize::new(0),
                    send_failures: AtomicUsize::new(0),
                };

                #[inline]
                fn evaluate_diagnostics(
                    result: Result<(), PostmasterError>,
                ) -> Result<(), PostmasterError> {
                    result
                        .inspect(|_| {
                            POSTMASTER.messages_sent.fetch_add(1, Ordering::Relaxed);
                        })
                        .inspect_err(|_| {
                            POSTMASTER.send_failures.fetch_add(1, Ordering::Relaxed);
                        })
                }
            }
        }
    };
}
