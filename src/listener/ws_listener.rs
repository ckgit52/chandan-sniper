use tokio_tungstenite::connect_async;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::tungstenite::Message;
use tokio::time::{sleep, Duration};

use crate::listener::mint_extractor::analyze_transaction;

/// Toggle debug prints
const DEBUG_MODE: bool = true;

/// Helius WebSocket API key
const HELIUS_API_KEY: &str = "276d33c2-1fc1-48e0-9d1a-6ecbf4d31ab5";

/// SPL Token Program ID
const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

/// Start the WebSocket listener with robust reconnect
pub async fn start_listener() {
    let mut reconnect_attempt = 0;

    loop {
        if reconnect_attempt > 0 {
            let wait_secs = (reconnect_attempt * 5).min(60); // exponential backoff up to 60s
            println!("⏳ Reconnecting in {} seconds...", wait_secs);
            sleep(Duration::from_secs(wait_secs)).await;
        }

        reconnect_attempt += 1;

        match start_listener_inner().await {
            Ok(_) => {
                println!("✅ WebSocket exited cleanly");
                reconnect_attempt = 0; // reset on success
            }
            Err(e) => {
                println!("❌ WebSocket crashed: {:?}\n", e);
            }
        }
    }
}

/// Inner listener logic
async fn start_listener_inner() -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("wss://mainnet.helius-rpc.com/?api-key={}", HELIUS_API_KEY);

    println!("🔌 Connecting to WebSocket...");
    let (ws_stream, _) = connect_async(url).await?;
    println!("✅ Connected");

    let (mut write, mut read) = ws_stream.split();

    // Subscribe to all logs
    let sub_msg = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "logsSubscribe",
        "params": [
            {
                "filter": "all"
            },
            {
                "commitment": "confirmed"
            }
        ]
    });

    write.send(Message::Text(sub_msg.to_string())).await?;
    println!("🚀 Listening for all transaction logs...\n");

    while let Some(msg) = read.next().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                println!("⚠️ WS read error: {:?}", e);
                return Err(Box::new(e));
            }
        };

        let text = match msg.into_text() {
            Ok(t) => t,
            Err(_) => continue,
        };

        let v: serde_json::Value = match serde_json::from_str(&text) {
            Ok(j) => j,
            Err(_) => {
                if DEBUG_MODE {
                    println!("⚠️ Failed to parse JSON: {}", text);
                }
                continue;
            }
        };

        // Ignore subscription confirmation
        if v.get("result").is_some() && v.get("method").is_none() {
            if DEBUG_MODE {
                println!("📡 Subscription confirmed with id: {}", v["result"]);
            }
            continue;
        }

        // Process transaction logs
        if let Some(params) = v.get("params") {
            if let Some(result) = params.get("result") {
                if let Some(signature) = result.get("signature").and_then(|s| s.as_str()) {
                    if DEBUG_MODE {
                        println!("📩 Transaction log detected: {}", signature);
                    }

                    // Only analyze if transaction touches the Token Program
                    if let Some(logs) = result.get("logs").and_then(|l| l.as_array()) {
                        let token_program_hit = logs.iter().any(|log| {
                            log.as_str().map_or(false, |s| s.contains(TOKEN_PROGRAM_ID))
                        });

                        if token_program_hit {
                            let sig_clone = signature.to_string();
                            tokio::spawn(async move {
                                analyze_transaction(sig_clone).await;
                            });
                        } else if DEBUG_MODE {
                            println!("ℹ️ Transaction does not involve Token Program: {}", signature);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}