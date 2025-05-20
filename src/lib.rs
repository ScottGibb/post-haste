#![no_std]

pub mod agent;
pub mod error;

pub use error::PostmasterError;

#[macro_export]
macro_rules! init_postmaster {
    ($address_enum:ty, $payload_enum:ty) => {
        mod postmaster {
            use super::{Addresses, $payload_enum};
            use embassy_sync::channel::DynamicSender;
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
            //     address: Addresses,
            //     mailbox: DynamicSender<'static, $payload_enum>,
            // ) -> Result<(), PostmasterError> {
            //     unsafe { postmaster_internal::register_agent(address, mailbox) }
            // }

            pub struct Message {
                source: $address_enum,
                payload: $payload_enum,
            }

            mod postmaster_internal {
                use super::{ADDRESS_COUNT, Addresses, Message, PostmasterError, $payload_enum};
                use embassy_sync::blocking_mutex::raw::NoopRawMutex;
                use embassy_sync::channel::DynamicSender;
                use embassy_sync::mutex::Mutex;

                // pub(super) unsafe fn register_agent(
                //     address: Addresses,
                //     mailbox: DynamicSender<'static, $payload_enum>,
                // ) -> Result<(), PostmasterError> {
                //     if POSTMASTER.senders[address as usize].is_none() {
                //         POSTMASTER.senders[address as usize].replace(mailbox);
                //         Ok(())
                //     } else {
                //         Err(PostmasterError::AddressAlreadyTaken)
                //     }
                // }

                struct Postmaster<'a> {
                    senders:
                        Mutex<NoopRawMutex, [Option<DynamicSender<'a, Message>>; ADDRESS_COUNT]>,
                }

                unsafe impl Sync for Postmaster<'_> {}

                static POSTMASTER: Postmaster = Postmaster {
                    senders: Mutex::new([None; ADDRESS_COUNT]),
                };
            }
        }
    };
}

pub trait PostAddress {
    const VARIANT_COUNT: usize;
}
