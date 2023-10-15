mod controllers;
#[tokio::main]
async fn main() {
    controllers::run().await.unwrap_or_else(|err| eprintln!("{}", err));
}
