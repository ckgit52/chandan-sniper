use tokio::signal;

mod listener;

// use listener::logs_listener::start_listener;//
use listener::ws_listener::start_listener;


#[tokio::main]
async fn main() {
    println!("🚀 Sniper bot starting...");

    // start listener
    tokio::spawn(async {
        start_listener().await;
    });

    println!("🚀 Listening for new slots (blocks)...");

    // Keep program alive
    signal::ctrl_c().await.unwrap();
    println!("🛑 Shutting down...");
}
