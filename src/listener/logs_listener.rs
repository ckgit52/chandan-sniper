use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;

use crate::listener::mint_extractor::analyze_transaction;

pub async fn start_listener() {
    let ws_url = "wss://api.mainnet-beta.solana.com";

    println!("📡 Connecting...");

    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("❌ Failed to connect");

    println!("✅ Connected");

    let (mut write, mut read) = ws_stream.split();

    let subscribe_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "logsSubscribe",
        "params": [
            "all",
            { "commitment": "confirmed" }
        ]
    });

    write.send(Message::Text(subscribe_msg.to_string()))
        .await
        .unwrap();

    println!("🚀 Listening + analyzing transactions...");

    while let Some(msg) = read.next().await {
        if let Ok(Message::Text(text)) = msg {

            if text.contains("logsNotification") {
                println!("📡 Incoming log..."); // 👈 DEBUG LINE

                if let Some(sig) = extract_signature(&text) {
                    analyze_transaction(&sig).await;
                }
            }
        }
    }
}

fn extract_signature(text: &str) -> Option<String> {
    if let Some(start) = text.find("\"signature\":\"") {
        let start = start + 13;
        if let Some(end) = text[start..].find('"') {
            return Some(text[start..start + end].to_string());
        }
    }
    None
}
