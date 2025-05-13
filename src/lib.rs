#![no_std]

pub mod agent;
pub mod postmaster;

pub use variant_count::VariantCount;

#[macro_export]
macro_rules! init_postmaster {
    (Addresses: {$($address:ident),*}, Messages: $message_type:ty) => {
        #[derive($crate::VariantCount)]
        enum Address {
            $($address),*
        }

        mod postmaster_internal {
            use embassy_sync::channel::DynamicSender;
            use super::{Address, $message_type};

            struct Postmaster<'a> {
                senders:
                    [Option<DynamicSender<'a, <Self as PostmasterTypes>::Message>>; Address::VARIANT_COUNT],
            }

            unsafe impl Sync for Postmaster<'_> {}

            trait PostmasterTypes {
                type Message;
            }

            impl PostmasterTypes for Postmaster<'_> {
                type Message = $message_type;
            }

            static POSTMASTER: Postmaster = Postmaster {
                senders: [None; Address::VARIANT_COUNT],
            };
        }
    };
}

pub trait PostAddress {
    const VARIANT_COUNT: usize;
}
