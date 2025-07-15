#[cfg(target_os = "none")]
use embassy_sync::channel::DynamicReceiver as Receiver;
#[cfg(not(target_os = "none"))]
use tokio::sync::mpsc::Receiver;

#[cfg(target_os = "none")]
pub type Inbox<T> = Receiver<'static, T>;
#[cfg(not(target_os = "none"))]
pub type Inbox<T> = Receiver<T>;

#[allow(async_fn_in_trait)]
pub trait Agent {
    type Address;
    type Message;
    type Config;

    async fn create(address: Self::Address, config: Self::Config) -> Self;

    async fn run(self, inbox: Inbox<Self::Message>) -> !;
}
