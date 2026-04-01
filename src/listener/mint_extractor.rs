use reqwest::Client;
use serde_json::Value;
use tokio::time::{sleep, Duration};

/// Analyze a transaction by its signature
pub async fn analyze_transaction(signature: String) {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = Client::new();

    // Retry up to 5 times
    for attempt in 0..5 {
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

        let resp = client.post(rpc_url).json(&body).send().await;

        if resp.is_err() {
            eprintln!("⚠️ Attempt {}: RPC request failed for {}", attempt + 1, signature);
            sleep(Duration::from_millis(500)).await;
            continue;
        }

        let json: Value = resp.unwrap().json().await.unwrap();

        if json["result"].is_null() {
            if cfg!(debug_assertions) {
                println!("ℹ️ Attempt {}: Transaction not found yet: {}", attempt + 1, signature);
            }
            sleep(Duration::from_millis(500)).await;
            continue;
        }

        let instructions = &json["result"]["transaction"]["message"]["instructions"];

        if instructions.is_null() {
            if cfg!(debug_assertions) {
                println!("ℹ️ No instructions in transaction: {}", signature);
            }
            return;
        }

        let mut new_mint: Option<String> = None;

        if let Some(ixs) = instructions.as_array() {
            println!("📌 Inspecting {} instructions for transaction {}", ixs.len(), signature);

            for (idx, ix) in ixs.iter().enumerate() {
                if let Some(parsed) = ix.get("parsed") {
                    if let Some(t) = parsed.get("type").and_then(|v| v.as_str()) {
                        if t == "initializeMint" || t == "initializeMint2" {
                            if let Some(mint) = parsed["info"]["mint"].as_str() {
                                new_mint = Some(mint.to_string());
                                println!(
                                    "🎯 Instruction {}: Detected new mint {} (type: {})",
                                    idx, mint, t
                                );
                            }
                        } else if cfg!(debug_assertions) {
                            println!(
                                "ℹ️ Instruction {}: Type {} (not a mint)",
                                idx, t
                            );
                        }
                    }
                } else if cfg!(debug_assertions) {
                    println!("ℹ️ Instruction {} has no parsed field", idx);
                }
            }
        }

        if let Some(mint) = new_mint {
            println!("\n==============================");
            println!("🚨 NEW TOKEN DETECTED");
            println!("🪙 Mint: {}", mint);

            // 🔥 Fetch metadata
            fetch_token_metadata(&client, &mint).await;

            println!("==============================\n");
        } else if cfg!(debug_assertions) {
            println!("ℹ️ No mint instructions found in transaction: {}", signature);
        }

        return;
    }
}

/// Fetch token metadata from Dexscreener
pub async fn fetch_token_metadata(client: &Client, mint: &str) {
    let url = format!("https://api.dexscreener.com/latest/dex/tokens/{}", mint);

    let resp = client.get(&url).send().await;

    if resp.is_err() {
        println!("⚠️ Metadata fetch failed for {}", mint);
        return;
    }

    let json: Value = resp.unwrap().json().await.unwrap();

    if json["pairs"].is_null() || json["pairs"].as_array().unwrap_or(&vec![]).is_empty() {
        println!("⚠️ No metadata found yet for mint {}", mint);
        return;
    }

    let pair = &json["pairs"][0];

    let name = pair["baseToken"]["name"].as_str().unwrap_or("N/A");
    let symbol = pair["baseToken"]["symbol"].as_str().unwrap_or("N/A");
    let price = pair["priceUsd"].as_str().unwrap_or("N/A");

    println!("📛 Name: {}", name);
    println!("🔤 Symbol: {}", symbol);
    println!("💲 Price: {}", price);
}