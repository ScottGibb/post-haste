#![no_std]
#![feature(variant_count)]

use post_haste::init_postmaster;

enum Payloads {}

#[derive(Clone, Copy)]
enum Addresses {}

init_postmaster!(Addresses, Payloads);
