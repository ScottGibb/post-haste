use embassy_sync::channel::{DynamicReceiver};

pub type Inbox<T> = DynamicReceiver<'static, T>;

#[allow(async_fn_in_trait)]
pub trait Agent {
    type Address;
    type Message;
    type Config;

    async fn create(address: Self::Address, config: Self::Config) -> Self;

    async fn run(self, inbox: Inbox<Self::Message>) -> !;
}
