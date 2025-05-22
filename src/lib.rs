#![no_std]

pub mod agent;
pub mod error;

pub mod embassy {
    pub use embassy_executor::{task, Spawner};
    pub use embassy_sync::{
        blocking_mutex::raw::NoopRawMutex, channel::DynamicSender, mutex::Mutex,
    };
    pub use embassy_time::{Duration, Timer, WithTimeout};
}
pub use error::PostmasterError;

#[macro_export]
macro_rules! init_postmaster {
    ($address_enum:ty, $payload_enum:ty) => {
        mod postmaster {
            use super::{$address_enum, $payload_enum};
            use post_haste::PostmasterError;

            const ADDRESS_COUNT: usize = core::mem::variant_count::<$address_enum>();

            // #[macro_export]
            // macro_rules! register_agent {
            //     ($spawner: ident, $agent_address:ident, $agent:ty, $config:ident) => {{
            //         use embassy_sync::channel::Channel;
            //         static CHANNEL: Channel = Channel::new();

            //         $spawner.must_spawn($agent::new(CHANNEL.receiver, $config));
            //         unsafe { postmaster_internals::register_agent($agent_address, CHANNEL.sender) }
            //     }};
            // }

            // pub fn register_mailbox(
            //     address: $address_enum,
            //     mailbox: DynamicSender<'static, $payload_enum>,
            // ) -> Result<(), PostmasterError> {
            //     unsafe { postmaster_internal::register_agent(address, mailbox) }
            // }

            pub struct Message {
                source: $address_enum,
                payload: $payload_enum,
            }

            mod postmaster_internal {
                use super::{ADDRESS_COUNT, $address_enum, Message, PostmasterError, $payload_enum};
                use core::cell::RefCell;
                use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
                use post_haste::embassy;
                use embassy::WithTimeout;

                pub(super) async fn send_internal(
                    destination: $address_enum,
                    message: Message,
                    timeout: Option<embassy::Duration>,
                ) -> Result<(), PostmasterError> {
                    let timeout = match timeout {
                        Some(duration) => duration,
                        None => embassy::Duration::from_micros(
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

                #[embassy::task]
                pub(super) async fn delayed_send(destination: $address_enum, message: Message, delay: embassy::Duration, timeout: Option<embassy::Duration>) {
                    embassy::Timer::after(delay).await;
                    let source = message.source;
                    match send_internal(destination, message, timeout).await {
                        Ok(_) => (),
                        Err(error) => report_send_error(source, error).await,
                    }
                }

                struct Postmaster<'a> {
                    senders:
                        embassy::Mutex<embassy::NoopRawMutex, [Option<embassy::DynamicSender<'a, Message>>; ADDRESS_COUNT]>,
                    timeout_us: AtomicU32,
                    spawner: RefCell<Option<embassy::Spawner>>,
                    messages_sent: AtomicUsize,
                    send_failures: AtomicUsize,
                }

                unsafe impl Sync for Postmaster<'_> {}

                static POSTMASTER: Postmaster = Postmaster {
                    senders: embassy::Mutex::new([None; ADDRESS_COUNT]),
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

                async fn report_send_error(destination: $address_enum, error: PostmasterError) {
                    let error_report
                }
            }
        }
    };
}
