#![no_std]
#![feature(variant_count)]

#[derive(Clone, PartialEq, Debug)]
enum MessagePayload {}

post_haste::init_postmaster! {
    Addresses: {
        One,
        Two,
    },

    MessageCategories: {
        MessagePayload,
    }
}

fn main() {}
