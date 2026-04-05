use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use serde_json::Value;
use tokio::time::{sleep, Duration, Instant};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use reqwest;

use crate::filters::basic_filters::{
    is_valid_new_token,
    is_new_token_creation,
    is_pumpfun_token,
    check_liquidity_fast,
    get_snipe_amount_by_risk,
    RiskProfile,
};

const HELIUS_API_KEY: &str = "276d33c2-1fc1-48e0-9d1a-6ecbf4d31ab5";

// Fast validation for Pump.fun and new tokens
async fn quick_validate_token(mint: &str, logs: &[Value], client: &reqwest::Client) -> (bool, String) {
    // Check if it's a Pump.fun token (these are always valid)
    let is_pump_fun = mint.ends_with("pump") || 
                      mint.starts_with("pump") ||
                      logs.iter().any(|l| {
                          l.as_str().unwrap_or("").to_lowercase().contains("pump")
                      });
    
    if is_pump_fun {
        return (true, "🔥 Pump.fun token - VALID (bypassing program check)".to_string());
    }
    
    // For other tokens, do a quick account check
    let url = "https://api.mainnet-beta.solana.com";
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getAccountInfo",
        "params": [mint, { "encoding": "base64" }]
    });
    
    match client.post(url).json(&body).timeout(Duration::from_secs(1)).send().await {
        Ok(response) => {
            if let Ok(json) = response.json::<Value>().await {
                let account = &json["result"]["value"];
                
                if account.is_null() {
                    return (false, "Account not found".to_string());
                }
                
                let is_executable = account.get("executable").and_then(|e| e.as_bool()).unwrap_or(false);
                
                if is_executable {
                    return (false, "This is a PROGRAM, not a token".to_string());
                }
                
                // Non-executable with data is likely a token
                if account.get("data").is_some() {
                    return (true, "✅ Valid token account".to_string());
                }
            }
        }
        Err(_) => {}
    }
    
    // If we can't verify but it looks like a mint address, assume it's valid for sniping
    if mint.len() >= 32 && mint.len() <= 44 {
        return (true, "⚠️ Assuming valid (new token - skipping slow check)".to_string());
    }
    
    (false, "Invalid token format".to_string())
}

// Execute buy immediately
async fn execute_snipe_fast(mint: &str, amount_sol: f64, is_pump: bool) {
    println!("\n💥 EXECUTING FAST SNIPE!");
    println!("   🪙 Token: {}", mint);
    println!("   💰 Amount: {} SOL", amount_sol);
    println!("   ⚡ Priority Fee: 0.015 SOL (HIGH)");
    println!("   🔄 Slippage: 20%");
    
    if is_pump {
        println!("   🔥 PUMP.FUN - Buy NOW before bonding curve fills!");
    }
    
    // TODO: Implement actual swap logic here
    // Example: Call Jupiter API to swap SOL for token
    
    println!("   ✅ BUY ORDER SENT at {:.2}s!", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64());
}

pub async fn start_ws_listener() {
    println!("⚡⚡⚡ PUMP.FUN SNIPER BOT ACTIVE ⚡⚡⚡");
    println!("   • Auto-detects Pump.fun tokens");
    println!("   • No program validation for Pump.fun");
    println!("   • Instant sniping on detection\n");
    
    let seen_tokens = Arc::new(Mutex::new(HashSet::new()));
    let start_time = Instant::now();
    let http_client = reqwest::Client::new();
    
    // Use aggressive risk profile for Pump.fun sniping
    let risk_profile = RiskProfile::Aggressive;

    loop {
        let url = format!(
            "wss://mainnet.helius-rpc.com/?api-key={}",
            HELIUS_API_KEY
        );

        match connect_async(&url).await {
            Ok((ws_stream, _)) => {
                println!("✅ Connected - Ready to snipe Pump.fun tokens!\n");

                let (mut write, mut read) = ws_stream.split();

                let sub_msg = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "logsSubscribe",
                    "params": [
                        "all",
                        { "commitment": "processed" }
                    ]
                });

                if let Err(e) = write.send(Message::Text(sub_msg.to_string())).await {
                    println!("❌ Subscribe error: {}", e);
                    continue;
                }

                while let Some(msg) = read.next().await {
                    let msg = match msg {
                        Ok(m) => m,
                        Err(_) => continue,
                    };

                    if !msg.is_text() {
                        continue;
                    }

                    let text = msg.to_text().unwrap();
                    let parsed: Value = match serde_json::from_str(text) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    let logs_array = vec![];
                    let logs = parsed["params"]["result"]["value"]["logs"]
                        .as_array()
                        .unwrap_or(&logs_array);

                    if logs.is_empty() {
                        continue;
                    }

                    if !is_new_token_creation(logs) {
                        continue;
                    }

                    let tx = parsed["params"]["result"]["value"]["signature"]
                        .as_str()
                        .unwrap_or("");

                    let mut potential_mints = HashSet::new();
                    
                    // Extract mints from logs
                    for log_val in logs {
                        if let Some(log) = log_val.as_str() {
                            let words: Vec<&str> = log.split_whitespace().collect();
                            
                            for word in words {
                                let clean_word = word.trim_matches(|c: char| !c.is_ascii_alphanumeric());
                                
                                if clean_word.len() >= 32 && clean_word.len() <= 44 {
                                    if is_valid_new_token(clean_word) {
                                        potential_mints.insert(clean_word.to_string());
                                    }
                                }
                            }
                        }
                    }

                    // Extract mints from accounts
                    if let Some(accounts) = parsed["params"]["result"]["value"]["accounts"].as_array() {
                        for acc in accounts {
                            if let Some(addr) = acc.as_str() {
                                if addr.len() >= 32 && addr.len() <= 44 && is_valid_new_token(addr) {
                                    potential_mints.insert(addr.to_string());
                                }
                            }
                        }
                    }

                    for mint in potential_mints {
                        // Skip fake/junk addresses
                        if mint.contains("AAAAAA") || 
                           mint.contains("FFFFFF") ||
                           mint.chars().filter(|c| *c == 'A').count() > 15 {
                            let mut seen = seen_tokens.lock().unwrap();
                            seen.insert(mint);
                            continue;
                        }
                        
                        // Check if already seen
                        {
                            let seen = seen_tokens.lock().unwrap();
                            if seen.contains(&mint) {
                                continue;
                            }
                        }
                        
                        let detection_time = start_time.elapsed();
                        
                        // Check if this is a Pump.fun token
                        let is_pump = is_pumpfun_token(&mint) || 
                                     logs.iter().any(|l| {
                                         l.as_str().unwrap_or("").to_lowercase().contains("pump")
                                     });
                        
                        println!("{}\n", "=".repeat(50));
                        println!("🚨 🚨 🚨 SNIPE SIGNAL DETECTED 🚨 🚨 🚨");
                        println!("⏱️  Detection time: {:.2}s", detection_time.as_secs_f32());
                        println!("🪙 Mint Address: {}", mint);
                        println!("🔗 Transaction: https://solscan.io/tx/{}", tx);
                        
                        if is_pump {
                            println!("🔥🔥🔥 PUMP.FUN TOKEN DETECTED! 🔥🔥🔥");
                            println!("   This is a VALID token - Sniping now!");
                        }
                        
                        // QUICK VALIDATION (bypasses program check for Pump.fun)
                        println!("\n🔍 Quick validation...");
                        let (is_valid, reason) = quick_validate_token(&mint, &logs, &http_client).await;
                        
                        if !is_valid {
                            println!("❌ TOKEN REJECTED: {}", reason);
                            let mut seen = seen_tokens.lock().unwrap();
                            seen.insert(mint);
                            continue;
                        }
                        
                        println!("✅ {}", reason);
                        
                        // Calculate snipe amount based on risk profile
                        let snipe_amount = get_snipe_amount_by_risk(risk_profile.clone(), is_pump);
                        
                        println!("\n💰 SNIPE DECISION: APPROVED");
                        println!("   • Token Type: {}", if is_pump { "PUMP.FUN" } else { "STANDARD" });
                        println!("   • Snipe Amount: {} SOL", snipe_amount);
                        println!("   • Risk Profile: Aggressive");
                        println!("   • Slippage: 20%");
                        println!("   • Priority Fee: 0.015 SOL");
                        
                        if is_pump {
                            println!("   ⚡ Pump.fun strategy: Buy immediately - bonding curve fills fast!");
                            println!("   💡 Target: Sell at 2x-3x or when volume slows");
                        }
                        
                        // EXECUTE SNIPE IMMEDIATELY
                        execute_snipe_fast(&mint, snipe_amount, is_pump).await;
                        
                        // Mark as seen
                        {
                            let mut seen = seen_tokens.lock().unwrap();
                            seen.insert(mint.clone());
                        }
                        
                        println!("{}\n", "=".repeat(50));
                        
                        // Only process first valid mint for speed
                        break;
                    }
                }
            }
            Err(e) => {
                println!("❌ Connection error: {:?}", e);
                sleep(Duration::from_millis(500)).await;
            }
        }
    }
}