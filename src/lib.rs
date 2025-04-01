#![no_std]

pub mod agent;
pub mod postmaster;

#[macro_export]
#[crabtime::function]
fn init_postmaster(pattern!($address_type:ty, $message_type:ty): _) {
    crabtime::output! {println!("Hello")}
}
