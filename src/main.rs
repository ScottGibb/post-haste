enum Address {}
enum Message {}
fn main() {
    post_haste::init_postmaster!(Address, Message);
}
