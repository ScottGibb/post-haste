#![no_std]
#![feature(variant_count)]

use embassy_executor::Spawner;
use post_haste::init_postmaster;

mod polite_agent;

use polite_agent::PoliteAgent;

enum Payloads {
    Hello,
}

#[derive(Debug, Clone, Copy)]
enum Addresses {
    A,
    B,
}

init_postmaster!(Addresses, Payloads);

pub async fn run(spawner: Spawner) {
    register_agent!(spawner, A, PoliteAgent, ()).unwrap();
    register_agent!(spawner, B, PoliteAgent, (), 2).unwrap();

    postmaster::send(Addresses::A, Addresses::B, Payloads::Hello).await.unwrap();
}
