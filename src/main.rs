#![no_std]
#![feature(variant_count)]

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Addresses {}

#[derive(Clone, Debug, PartialEq)]
enum MessagePayload {}

post_haste::init_postmaster!(Addresses, MessagePayload);

fn main() {}
