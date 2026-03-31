use reqwest::Client;
use serde_json::Value;
use tokio::time::{sleep, Duration};

pub async fn analyze_transaction(signature: &str) {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = Client::new();

    for _ in 0..5 {
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
            return;
        }

        let json: Value = resp.unwrap().json().await.unwrap();

        if json["result"].is_null() {
            sleep(Duration::from_millis(500)).await;
            continue;
        }

        let instructions = &json["result"]["transaction"]["message"]["instructions"];

        let mut new_mint: Option<String> = None;

        if let Some(ixs) = instructions.as_array() {
            for ix in ixs {
                if let Some(parsed) = ix.get("parsed") {
                    if let Some(t) = parsed.get("type") {
                        let t = t.as_str().unwrap_or("");

                        if t == "initializeMint" || t == "initializeMint2" {
                            if let Some(mint) = parsed["info"]["mint"].as_str() {
                                new_mint = Some(mint.to_string());
                            }
                        }
                    }
                }
            }
        }

        if let Some(mint) = new_mint {
            println!("\n==============================");
            println!("🚨 NEW TOKEN DETECTED");
            println!("🪙 Mint: {}", mint);

            // 🔥 Fetch metadata
            fetch_token_metadata(&client, &mint).await;

            println!("==============================");
        }

        return;
    }
}

// 🚀 Fetch metadata from Solana (Metaplex)
async fn fetch_token_metadata(client: &Client, mint: &str) {
    let url = format!("https://api.dexscreener.com/latest/dex/tokens/{}", mint);

    let resp = client.get(&url).send().await;

    if resp.is_err() {
        println!("⚠️ Metadata fetch failed");
        return;
    }

    let json: Value = resp.unwrap().json().await.unwrap();

    if json["pairs"].is_null() {
        println!("⚠️ No metadata found yet");
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
