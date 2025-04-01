#![no_std]
#![feature(variant_count)]

pub mod agent;
pub mod postmaster;

pub mod prelude {
    pub use embassy_sync::channel::DynamicSender;
}

enum Address {
    One,
    Two,
}

enum Message {}
mod postmaster_internals {
    const NUM_ADDRESSES: usize = core::mem::variant_count::<crate::Address>();
    struct Postmaster<'a, MESSAGE> {
        senders: [Option<crate::prelude::DynamicSender<'a, MESSAGE>>; NUM_ADDRESSES],
    }

    static POSTMASTER: Postmaster<crate::Message> = Postmaster {
        senders: [None; NUM_ADDRESSES]
    }
}

macro_rules! use_post_haste {
    ($address: ty) => {
        mod postmaster_internals {
            struct Postmaster {
                senders: [post_haste::prelude::DynamicSender;]
            }
        }
    };
}
