#![no_std]
#![feature(variant_count)]
enum Addresses {}
enum MessagePayload {}

post_haste::init_postmaster!(Addresses, MessagePayload);

fn main() {}
