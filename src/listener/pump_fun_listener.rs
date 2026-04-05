use reqwest::Client;
use serde_json::Value;
use tokio::time::{sleep, Duration};
use std::collections::HashSet;

use crate::filters::basic_filters::filter_pump_tokens;

pub async fn start_pump_fun_listener() {
    let client = Client::new();
    let mut seen: HashSet<String> = HashSet::new();

    println!("🚀 Pump.fun listener started...\n");

    loop {
        let url = "https://api.dexscreener.com/latest/dex/search?q=pump";

        println!("🔄 Fetching data...");

        // ✅ Send request safely
        let resp = match client
            .get(url)
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                println!("⚠️ Request failed: {:?}", e);
                sleep(Duration::from_secs(3)).await;
                continue;
            }
        };

        // ✅ Read response safely
        let text = match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                println!("⚠️ Failed to read response: {:?}", e);
                sleep(Duration::from_secs(3)).await;
                continue;
            }
        };

        // 🔍 Debug: empty or blocked response
        if text.is_empty() {
            println!("⚠️ Empty response (maybe rate limited)");
            sleep(Duration::from_secs(3)).await;
            continue;
        }

        // ✅ Parse JSON safely
        let json: Value = match serde_json::from_str(&text) {
            Ok(j) => j,
            Err(e) => {
                println!("⚠️ JSON parse failed: {:?}", e);
                println!("🔎 Raw response: {}", text);
                sleep(Duration::from_secs(3)).await;
                continue;
            }
        };

        // ✅ Extract pairs
        let pairs = match json.get("pairs").and_then(|p| p.as_array()) {
            Some(p) => p,
            None => {
                println!("⚠️ No pairs found in response");
                sleep(Duration::from_secs(3)).await;
                continue;
            }
        };

        println!("📊 Total pairs fetched: {}", pairs.len());

        // 🔥 Apply your filter logic
        let filtered_tokens = filter_pump_tokens(pairs, &mut seen);

        if filtered_tokens.is_empty() {
            println!("😴 No valid pump tokens found (filtered)");
        }

        // ✅ Print filtered tokens
        for pair in filtered_tokens {
            let mint = pair["baseToken"]["address"].as_str().unwrap_or("");
            let name = pair["baseToken"]["name"].as_str().unwrap_or("N/A");
            let symbol = pair["baseToken"]["symbol"].as_str().unwrap_or("N/A");
            let price = pair["priceUsd"].as_str().unwrap_or("N/A");
            let liquidity = pair["liquidity"]["usd"].as_f64().unwrap_or(0.0);

            println!("\n==============================");
            println!("🚀 REAL PUMP TOKEN DETECTED");
            println!("🪙 Mint: {}", mint);
            println!("📛 Name: {}", name);
            println!("🔤 Symbol: {}", symbol);
            println!("💲 Price: {}", price);
            println!("💧 Liquidity: ${}", liquidity);
            println!("==============================\n");
        }

        // ⏳ Poll interval
        sleep(Duration::from_secs(3)).await;
    }
}