use embassy_sync::channel::DynamicSender;

enum Message {}

fn main() {}

post_haste::init_postmaster!(Addresses: {
        One,
        Two
    },
    Messages: Message);

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
