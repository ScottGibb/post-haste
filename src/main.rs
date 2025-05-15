enum Message {}

fn main() {}

post_haste::init_postmaster!(Addresses: {
        One,
        Two
    },
    Messages: Message);
