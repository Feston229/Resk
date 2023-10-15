// This is aka wrapper for parsing resk_node
// to flutter
use android_activity::AndroidApp;
use tokio::runtime::Runtime;

// Function for running tests via cargo apk run
#[no_mangle]
fn android_main(_app: AndroidApp) {
    // Logger
    android_logger::init_once(
        android_logger::Config::default().with_min_level(log::Level::Info),
    );
}

// Actuall function that is called from Flutter via ffi
#[no_mangle]
pub extern "C" fn run_node_android(flutter_udp_port: i32) {
    // Tokio runtime
    let rt = Runtime::new().unwrap();

    // Logger
    android_logger::init_once(
        android_logger::Config::default().with_min_level(log::Level::Info),
    );

    // Run
    rt.block_on(async {
        resk_node::controllers::run_node(Some(flutter_udp_port))
            .await
            .unwrap_or_else(|err| log::info!("{err}"));
    });
}
