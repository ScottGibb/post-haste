#[allow(async_fn_in_trait)]
pub trait Agent {
    type Address;
    type Config;

    async fn create(address: Self::Address, config: Self::Config) -> Self;

    async fn run(self) -> !;
}
