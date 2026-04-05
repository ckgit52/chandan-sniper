use reqwest::Client;
use serde_json::Value;

const HELIUS_API_KEY: &str = "276d33c2-1fc1-48e0-9d1a-6ecbf4d31ab5";

pub async fn process_fast_sniper(signature: String) {
    println!("⚡ Processing TX: {}", signature);

    let rpc_url = format!(
        "https://mainnet.helius-rpc.com/?api-key={}",
        HELIUS_API_KEY
    );

    let client = Client::new();

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getTransaction",
        "params": [
            signature,
            {
                "encoding": "jsonParsed",
                "maxSupportedTransactionVersion": 0
            }
        ]
    });

    let resp = client.post(&rpc_url).json(&body).send().await;

    if resp.is_err() {
        println!("❌ TX fetch failed");
        return;
    }

    let json: Value = resp.unwrap().json().await.unwrap();

    if json["result"].is_null() {
        println!("⏳ TX not ready yet");
        return;
    }

    let instructions = match json["result"]["transaction"]["message"]["instructions"].as_array() {
        Some(ix) => ix,
        None => return,
    };

    let mut detected_mint = None;

    for ix in instructions {
        if let Some(parsed) = ix.get("parsed") {
            if let Some(info) = parsed.get("info") {
                if let Some(mint) = info.get("mint").and_then(|m| m.as_str()) {
                    detected_mint = Some(mint.to_string());
                    break;
                }
            }
        }
    }

    if let Some(mint) = detected_mint {
        println!("💰 TOKEN FOUND: {}", mint);

        // 🚨 HERE is your real sniper entry
        execute_buy(mint).await;
    } else {
        println!("⚠️ No mint found");
    }
}

async fn execute_buy(mint: String) {
    println!("🚀 BUYING TOKEN: {}", mint);

    // 🔥 REAL IMPLEMENTATION:
    // - Use Jupiter API
    // - Use Raydium swap
    // - Add priority fee

    // For now:
    println!("💸 BUY EXECUTED (SIMULATION)");
}