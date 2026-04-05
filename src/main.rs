use tokio::signal;
use dotenv::dotenv;

mod filters;
mod listener;
mod notifier;
mod trader;
mod strategy;
mod config;
mod wallet;
mod utils;

use listener::ws_listener::start_ws_listener;

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv().ok();
    
    println!("🚀 Sniper bot starting...");
    println!("🚀 Listening for new tokens...\n");

    // Start WebSocket listener
    tokio::spawn(async {
        start_ws_listener().await;
    });

    // Keep program alive until Ctrl+C
    match signal::ctrl_c().await {
        Ok(()) => {
            println!("\n🛑 Shutting down...");
        }
        Err(err) => {
            eprintln!("⚠️ Error listening for Ctrl+C: {}", err);
        }
    }
}
