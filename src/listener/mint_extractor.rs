use reqwest::Client;
use serde_json::Value;
use tokio::time::{sleep, Duration};

pub async fn analyze_transaction(signature: String) {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = Client::new();

    for _ in 0..6 {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTransaction",
            "params": [
                signature,
                {
                    "encoding": "json",
                    "maxSupportedTransactionVersion": 0
                }
            ]
        });

        let resp = client.post(rpc_url).json(&body).send().await;

        if resp.is_err() {
            sleep(Duration::from_millis(1000)).await;
            continue;
        }

        let json: Value = resp.unwrap().json().await.unwrap();

        if json["result"].is_null() {
            sleep(Duration::from_millis(1000)).await;
            continue;
        }

        let tx = &json["result"]["transaction"]["message"];

        let empty_vec = vec![];
        let account_keys = tx["accountKeys"].as_array().unwrap_or(&empty_vec);

        let empty_vec = vec![];
        let instructions = tx["instructions"].as_array().unwrap_or(&empty_vec);

        let mut found = false;

        for ix in instructions {
            // programIdIndex tells which program is used
            let program_idx = ix["programIdIndex"].as_u64().unwrap_or(999) as usize;

            if program_idx >= account_keys.len() {
                continue;
            }

            let program = account_keys[program_idx].as_str().unwrap_or("");

            // Token Program check
            if program != "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" {
                continue;
            }

            // 🔥 For InitializeMint, mint account is usually first account
            if let Some(accounts) = ix["accounts"].as_array() {
                if !accounts.is_empty() {
                    let mint_idx = accounts[0].as_u64().unwrap_or(999) as usize;

                    if mint_idx < account_keys.len() {
                        let mint = account_keys[mint_idx].as_str().unwrap_or("N/A");

                        println!("\n==============================");
                        println!("🚨 NEW TOKEN DETECTED (RAW)");
                        println!("🔗 Signature: {}", signature);
                        println!("🪙 Mint: {}", mint);
                        println!("==============================\n");

                        found = true;
                    }
                }
            }
        }

        if !found {
            println!("⚠️ Could not decode mint (raw fallback): {}", signature);
        }

        return;
    }
}