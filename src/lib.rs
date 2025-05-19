#![no_std]

pub mod agent;
pub mod error;

pub use error::PostmasterError;
pub use variant_count::VariantCount;

#[macro_export]
macro_rules! init_postmaster {
    ($address_enum:ty, $message_enum:ty) => {
        mod postmaster {
            #![feature(variant_count)]
            const ADDRESS_COUNT: usize = core::mem::variant_count::<$address_enum>();
            use super::{Addresses, $message_enum};
            use embassy_sync::channel::DynamicSender;
            use post_haste::PostmasterError;

            macro_rules! register_agent {
                ($spawner: ident, $agent_address:ident, $agent:ty, $config:ident) => {{
                    use embassy_sync::channel::Channel;
                    static CHANNEL: Channel = Channel::new();

                    $spawner.must_spawn($agent::new(CHANNEL.receiver, $config));
                    unsafe { postmaster_internals::register_agent($agent_address, CHANNEL.sender) }
                }};
            }

            pub fn register_mailbox(
                address: Addresses,
                mailbox: DynamicSender<'static, $message_enum>,
            ) -> Result<(), PostmasterError> {
                unsafe { postmaster_internal::register_agent(address, mailbox) }
            }

            mod postmaster_internal {
                use super::{ADDRESS_COUNT, Addresses, PostmasterError, $message_enum};
                use embassy_sync::channel::DynamicSender;

                pub(super) unsafe fn register_agent(
                    address: Addresses,
                    mailbox: DynamicSender<'static, $message_enum>,
                ) -> Result<(), PostmasterError> {
                    if POSTMASTER.senders[address as usize].is_none() {
                        POSTMASTER.senders[address as usize].replace(mailbox);
                        Ok(())
                    } else {
                        Err(PostmasterError::AddressAlreadyTaken)
                    }
                }

                struct Postmaster<'a> {
                    senders: [Option<DynamicSender<'a, <Self as PostmasterTypes>::Message>>;
                        ADDRESS_COUNT],
                }

                unsafe impl Sync for Postmaster<'_> {}

                trait PostmasterTypes {
                    type Message;
                }

                impl PostmasterTypes for Postmaster<'_> {
                    type Message = $message_enum;
                }

                static mut POSTMASTER: Postmaster = Postmaster {
                    senders: [None; Addresses::VARIANT_COUNT],
                };
            }
        }
    };
}

pub trait PostAddress {
    const VARIANT_COUNT: usize;
}
