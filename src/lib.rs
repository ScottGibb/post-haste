#![no_std]

pub mod agent;
pub mod postmaster;

#[macro_export]
macro_rules! init_postmaster {
    ($address:ty, $message:ty) => {
        mod postmaster_internal {
            use embassy_sync::channel::DynamicSender;

            const ADDRESS_COUNT: usize = core::mem::variant_count::<$address>();

            struct Postmaster<'a> {
                senders:
                    [Option<DynamicSender<'a, <Self as PostmasterTypes>::Message>>; ADDRESS_COUNT],
            }

            unsafe impl Sync for Postmaster<'_> {}

            trait PostmasterTypes {
                type Address;
                type Message;
            }

            impl PostmasterTypes for Postmaster<'_> {
                type Address = $address;
                type Message = $message;
            }

            static POSTMASTER: Postmaster = Postmaster {
                senders: [None; ADDRESS_COUNT],
            };
        }
    };
}
