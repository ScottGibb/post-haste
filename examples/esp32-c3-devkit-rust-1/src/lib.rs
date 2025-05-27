#![no_std]
#![feature(variant_count)]

use post_haste::init_postmaster;

enum Payloads {}

#[derive(Clone)]
enum Addresses {}

init_postmaster!(Addresses, Payloads);
