#![feature(variant_count)]
use embassy_sync::channel::DynamicSender;

enum Address {}

enum Message {}
fn main() {
    post_haste::init_postmaster!(crate::Address, crate::Message);
}

struct Postmaster<'a> {
    senders: [DynamicSender<'a, <Postmaster<'a> as Pm>::Message>],
}

trait Pm {
    type Address;
    type Message;
}

impl Pm for Postmaster<'_> {
    type Address = Address;
    type Message = Message;
}
