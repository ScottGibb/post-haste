#![no_std]

pub mod agent;
pub mod error;

pub use error::PostmasterError;
pub use variant_count::VariantCount;

#[macro_export]
macro_rules! init_postmaster {
    (Addresses: {$($address:ident),*}, Messages: $message_type:ty) => {
        #[derive($crate::VariantCount, Clone, Copy)]
        enum Addresses {
            $($address),*
        }

        mod postmaster {
            use embassy_sync::channel::DynamicSender;
            use super::{Addresses, $message_type};
            use post_haste::PostmasterError;

            macro_rules! register_agent {
                ($spawner: ident, $agent_address:ident, $agent:ty, $config:ident) => {
                    {
                        use embassy_sync::channel::Channel;
                        static CHANNEL: Channel = Channel::new();

                        $spawner.must_spawn($agent::new(CHANNEL.receiver, $config));
                        unsafe { postmaster_internals::register_agent($agent_address, CHANNEL.sender) }
                    }
                }
            }

            pub fn register_mailbox(address: Addresses, mailbox: DynamicSender<'static, $message_type>) -> Result<(), PostmasterError> {unsafe { postmaster_internal::register_agent(address, mailbox) }

            }

            mod postmaster_internal {
                use embassy_sync::channel::DynamicSender;
                use super::{Addresses, $message_type, PostmasterError};

                pub(super) unsafe fn register_agent(address: Addresses, mailbox: DynamicSender<'static, $message_type>) -> Result<(), PostmasterError> {
                    if POSTMASTER.senders[address as usize].is_none() {
                    POSTMASTER.senders[address as usize].replace(mailbox);
                    Ok(())
                    } else {
                        Err(PostmasterError::AddressAlreadyTaken)
                    }
                }

                struct Postmaster<'a> {
                    senders:
                        [Option<DynamicSender<'a, <Self as PostmasterTypes>::Message>>; Addresses::VARIANT_COUNT],
                }

                unsafe impl Sync for Postmaster<'_> {}

                trait PostmasterTypes {
                    type Message;
                }

                impl PostmasterTypes for Postmaster<'_> {
                    type Message = $message_type;
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
