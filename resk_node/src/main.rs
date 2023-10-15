mod controllers;
#[cfg(any(target_os = "android", target_os = "ios"))]
mod mobile;
mod utils;

use controllers::run_node;

#[tokio::main]
async fn main() {
    run_node(None)
        .await
        .unwrap_or_else(|err| eprintln!("{err}"));
}
