use serde_json::Value;
use std::collections::HashSet;
use reqwest;
use tokio::time::{sleep, Duration, Instant};

const IGNORED_MINTS: [&str; 3] = [
    "So11111111111111111111111111111111111111112",
    "EPjFWdd5AufqSSqeM2qg4z8G4wG3KQ3vZcP9y4x8u4",
    "Es9vMFrzaCERmJfrF4H2h5X3QxuxMxDPZWS9Vyuk3F7",
];

#[derive(Debug, Clone)]
pub struct TokenSafety {
    pub is_safe: bool,
    pub reasons: Vec<String>,
    pub score: u8,
}

#[derive(Debug, Clone)]
pub struct ProfitabilityCheck {
    pub is_profitable: bool,
    pub expected_return: f64,
    pub risk_reward_ratio: f64,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DexScreenerInfo {
    pub is_listed: bool,
    pub price_usd: f64,
    pub liquidity_usd: f64,
    pub volume_24h: f64,
    pub market_cap: f64,
    pub price_change_5m: f64,
    pub price_change_1h: f64,
    pub url: String,
}

#[derive(Clone, Debug)]
pub enum RiskProfile {
    Extreme,    // Small test snipes (0.02-0.03 SOL)
    Aggressive, // Standard snipes (0.05-0.1 SOL)
    Balanced,   // Medium snipes (0.1-0.15 SOL)
    Safe,       // Larger snipes (0.15-0.2 SOL)
}

// ========== VALIDATION FUNCTIONS ==========

pub fn is_valid_mint_format(address: &str) -> bool {
    if address.len() < 32 || address.len() > 44 {
        return false;
    }
    
    // Reject obviously fake addresses
    if address.contains("AAAAAAAA") || 
       address.contains("FFFFFFF") ||
       address.chars().filter(|c| *c == 'A').count() > 20 ||
       address.chars().filter(|c| *c == 'G').count() > 20 {
        return false;
    }
    
    // Must have good entropy
    let unique_chars: HashSet<char> = address.chars().collect();
    if unique_chars.len() < 10 {
        return false;
    }
    
    // Valid base58 characters only
    address.chars().all(|c| {
        matches!(c, '1'..='9' | 'A'..='H' | 'J'..='N' | 'P'..='Z' | 'a'..='k' | 'm'..='z')
    })
}

pub fn is_valid_new_token(address: &str) -> bool {
    if address.len() < 32 || address.len() > 44 {
        return false;
    }
    
    if !is_valid_mint_format(address) {
        return false;
    }
    
    if IGNORED_MINTS.contains(&address) {
        return false;
    }
    
    let invalid_prefixes = [
        "1111111111", "Tokenkeg", "ATokenGP", "ComputeB", 
        "SysvarR", "Vote1111", "Stake111", "Config11", 
        "BPFLoader", "B3111yJ", "metaqbxx", "pAMM", "cpam", "FLASH",
    ];
    
    for prefix in invalid_prefixes {
        if address.starts_with(prefix) {
            return false;
        }
    }
    
    true
}

pub fn is_new_token_creation(logs: &[Value]) -> bool {
    for log_val in logs {
        if let Some(log) = log_val.as_str() {
            let log_lower = log.to_lowercase();
            
            if (log_lower.contains("initialize") && log_lower.contains("mint")) ||
               log.contains("Program log: Create") ||
               log.contains("initializeMint") ||
               (log_lower.contains("create") && log_lower.contains("token")) ||
               (log.contains("pump") && log.contains("create")) ||
               log_lower.contains("pump.fun") {
                return true;
            }
        }
    }
    false
}

pub fn is_pumpfun_token(mint: &str) -> bool {
    mint.ends_with("pump") || 
    mint.starts_with("pump") || 
    mint.contains("pump") ||
    mint.len() == 44 // Pump.fun tokens are usually full length
}

// ========== FAST CHECKS (0.5-1 second) ==========

pub async fn check_liquidity_fast(mint: &str) -> (bool, f64) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(500))
        .build()
        .unwrap();
    
    let url = format!("https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint=So11111111111111111111111111111111111111112&amount=1000000&slippageBps=100", mint);
    
    match client.get(&url).send().await {
        Ok(response) => {
            if let Ok(json) = response.json::<Value>().await {
                if let Some(price) = json.get("price").and_then(|p| p.as_f64()) {
                    let liquidity = price * 1000000.0;
                    return (true, liquidity);
                }
            }
        }
        Err(_) => {}
    }
    (false, 0.0)
}

pub async fn check_token_safety_fast(mint: &str, client: &reqwest::Client) -> (bool, u8) {
    let url = "https://api.mainnet-beta.solana.com";
    
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getAccountInfo",
        "params": [mint, { "encoding": "jsonParsed" }]
    });
    
    match client.post(url).json(&body).timeout(Duration::from_secs(1)).send().await {
        Ok(response) => {
            if let Ok(json) = response.json::<Value>().await {
                if let Some(parsed) = json["result"]["value"]["data"]["parsed"].as_object() {
                    if let Some(info) = parsed.get("info") {
                        let mint_authority = info.get("mintAuthority")
                            .and_then(|a| a.as_str())
                            .unwrap_or("");
                        
                        let freeze_authority = info.get("freezeAuthority")
                            .and_then(|a| a.as_str())
                            .unwrap_or("");
                        
                        let has_mint_auth = mint_authority != "11111111111111111111111111111111" && !mint_authority.is_empty();
                        let has_freeze_auth = freeze_authority != "11111111111111111111111111111111" && !freeze_authority.is_empty();
                        
                        if has_mint_auth || has_freeze_auth {
                            return (false, 30);
                        } else {
                            return (true, 90);
                        }
                    }
                }
            }
        }
        Err(_) => {}
    }
    (true, 50)
}

// ========== THOROUGH CHECKS (3-5 seconds) ==========

pub async fn check_token_safety(mint: &str, client: &reqwest::Client) -> TokenSafety {
    let mut safety = TokenSafety {
        is_safe: true,
        reasons: Vec::new(),
        score: 100,
    };
    
    match check_mint_authority(mint, client).await {
        Ok(has_mint_authority) => {
            if has_mint_authority {
                safety.is_safe = false;
                safety.reasons.push("❌ Mint authority still enabled".to_string());
                safety.score -= 35;
            } else {
                safety.reasons.push("✅ Mint authority renounced".to_string());
            }
        }
        Err(_) => {
            safety.reasons.push("❓ Could not verify mint authority".to_string());
            safety.score -= 10;
        }
    }
    
    match check_freeze_authority(mint, client).await {
        Ok(has_freeze_authority) => {
            if has_freeze_authority {
                safety.reasons.push("❌ Freeze authority enabled".to_string());
                safety.score -= 25;
            } else {
                safety.reasons.push("✅ Freeze authority disabled".to_string());
            }
        }
        Err(_) => {
            safety.reasons.push("❓ Could not verify freeze authority".to_string());
            safety.score -= 5;
        }
    }
    
    match check_token_supply(mint, client).await {
        Ok((supply, decimals)) => {
            let adjusted_supply = supply / 10f64.powi(decimals as i32);
            
            if adjusted_supply > 1_000_000_000.0 {
                safety.reasons.push(format!("⚠️ Very high supply: {:.0}", adjusted_supply));
                safety.score -= 15;
            } else if adjusted_supply < 10_000.0 {
                safety.reasons.push(format!("⚠️ Very low supply: {:.0}", adjusted_supply));
                safety.score -= 10;
            } else {
                safety.reasons.push(format!("✅ Reasonable supply: {:.0}", adjusted_supply));
            }
        }
        Err(_) => {
            safety.reasons.push("❓ Could not fetch supply".to_string());
            safety.score -= 5;
        }
    }
    
    safety.is_safe = safety.score >= 50;
    safety
}

async fn check_mint_authority(mint: &str, client: &reqwest::Client) -> Result<bool, Box<dyn std::error::Error>> {
    let url = "https://api.mainnet-beta.solana.com";
    
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getAccountInfo",
        "params": [mint, { "encoding": "jsonParsed" }]
    });
    
    let response = client.post(url).json(&body).send().await?;
    let json: Value = response.json().await?;
    
    if let Some(parsed) = json["result"]["value"]["data"]["parsed"].as_object() {
        if let Some(info) = parsed.get("info") {
            let mint_authority = info.get("mintAuthority")
                .and_then(|a| a.as_str())
                .unwrap_or("");
            
            Ok(mint_authority != "11111111111111111111111111111111" && !mint_authority.is_empty())
        } else {
            Ok(true)
        }
    } else {
        Ok(true)
    }
}

async fn check_freeze_authority(mint: &str, client: &reqwest::Client) -> Result<bool, Box<dyn std::error::Error>> {
    let url = "https://api.mainnet-beta.solana.com";
    
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getAccountInfo",
        "params": [mint, { "encoding": "jsonParsed" }]
    });
    
    let response = client.post(url).json(&body).send().await?;
    let json: Value = response.json().await?;
    
    if let Some(parsed) = json["result"]["value"]["data"]["parsed"].as_object() {
        if let Some(info) = parsed.get("info") {
            let freeze_authority = info.get("freezeAuthority")
                .and_then(|a| a.as_str())
                .unwrap_or("");
            
            Ok(freeze_authority != "11111111111111111111111111111111" && !freeze_authority.is_empty())
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

async fn check_token_supply(mint: &str, client: &reqwest::Client) -> Result<(f64, u8), Box<dyn std::error::Error>> {
    let url = "https://api.mainnet-beta.solana.com";
    
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getTokenSupply",
        "params": [mint]
    });
    
    let response = client.post(url).json(&body).send().await?;
    let json: Value = response.json().await?;
    
    if let Some(supply_data) = json["result"]["value"].as_object() {
        let supply = supply_data["amount"]
            .as_str()
            .unwrap_or("0")
            .parse::<f64>()
            .unwrap_or(0.0);
        
        let decimals = supply_data["decimals"]
            .as_u64()
            .unwrap_or(9) as u8;
        
        Ok((supply, decimals))
    } else {
        Err("Could not fetch supply".into())
    }
}

// ========== DEXSCREENER INTEGRATION ==========

pub async fn check_dexscreener(mint: &str) -> DexScreenerInfo {
    let client = reqwest::Client::new();
    let url = format!("https://api.dexscreener.com/latest/dex/token/{}", mint);
    
    let mut info = DexScreenerInfo {
        is_listed: false,
        price_usd: 0.0,
        liquidity_usd: 0.0,
        volume_24h: 0.0,
        market_cap: 0.0,
        price_change_5m: 0.0,
        price_change_1h: 0.0,
        url: format!("https://dexscreener.com/solana/{}", mint),
    };
    
    match client.get(&url).timeout(Duration::from_secs(3)).send().await {
        Ok(response) => {
            if let Ok(json) = response.json::<Value>().await {
                if let Some(pairs) = json.get("pairs").and_then(|p| p.as_array()) {
                    if let Some(first_pair) = pairs.first() {
                        info.is_listed = true;
                        info.price_usd = first_pair.get("priceUsd")
                            .and_then(|p| p.as_str())
                            .and_then(|p| p.parse::<f64>().ok())
                            .unwrap_or(0.0);
                        
                        info.liquidity_usd = first_pair.get("liquidity")
                            .and_then(|l| l.get("usd"))
                            .and_then(|l| l.as_f64())
                            .unwrap_or(0.0);
                        
                        info.volume_24h = first_pair.get("volume")
                            .and_then(|v| v.get("h24"))
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);
                        
                        info.market_cap = first_pair.get("marketCap")
                            .and_then(|m| m.as_f64())
                            .unwrap_or(0.0);
                        
                        info.price_change_5m = first_pair.get("priceChange")
                            .and_then(|p| p.get("m5"))
                            .and_then(|p| p.as_f64())
                            .unwrap_or(0.0);
                        
                        info.price_change_1h = first_pair.get("priceChange")
                            .and_then(|p| p.get("h1"))
                            .and_then(|p| p.as_f64())
                            .unwrap_or(0.0);
                    }
                }
            }
        }
        Err(_) => {}
    }
    
    info
}

// ========== PROFITABILITY & AMOUNT CALCULATIONS ==========

pub async fn check_profitability(mint: &str, liquidity_usd: f64, safety_score: u8) -> ProfitabilityCheck {
    let mut check = ProfitabilityCheck {
        is_profitable: false,
        expected_return: 0.0,
        risk_reward_ratio: 0.0,
        reasons: Vec::new(),
    };
    
    if liquidity_usd >= 5000.0 {
        check.reasons.push(format!("✅ Good liquidity: ${:.0}", liquidity_usd));
    } else if liquidity_usd >= 1000.0 {
        check.reasons.push(format!("⚠️ Low liquidity: ${:.0}", liquidity_usd));
    } else {
        check.reasons.push(format!("❌ Very low liquidity: ${:.0}", liquidity_usd));
        return check;
    }
    
    if safety_score >= 80 {
        check.reasons.push(format!("✅ Good safety: {}/100", safety_score));
        check.expected_return = 50.0;
    } else if safety_score >= 60 {
        check.reasons.push(format!("⚠️ Moderate safety: {}/100", safety_score));
        check.expected_return = 25.0;
    } else {
        check.reasons.push(format!("❌ Poor safety: {}/100", safety_score));
        return check;
    }
    
    check.is_profitable = true;
    check.risk_reward_ratio = 2.5;
    check
}

pub fn calculate_snipe_amount(liquidity_usd: f64, safety_score: u8, expected_return: f64) -> f64 {
    let base_amount = 0.1;
    
    let liquidity_multiplier = if liquidity_usd > 50000.0 {
        2.0
    } else if liquidity_usd > 10000.0 {
        1.5
    } else if liquidity_usd > 5000.0 {
        1.0
    } else {
        0.5
    };
    
    let risk_multiplier = match safety_score {
        0..=20 => 0.2,
        21..=40 => 0.4,
        41..=60 => 0.7,
        61..=80 => 1.2,
        _ => 1.5,
    };
    
    let return_multiplier = if expected_return > 80.0 {
        1.5
    } else if expected_return > 50.0 {
        1.2
    } else {
        1.0
    };
    
    let amount: f64 = base_amount * liquidity_multiplier * risk_multiplier * return_multiplier;
    amount.min(2.0).max(0.05)
}

pub fn get_snipe_amount_by_risk(profile: RiskProfile, is_pumpfun: bool) -> f64 {
    match profile {
        RiskProfile::Extreme => {
            if is_pumpfun { 0.03 } else { 0.02 }
        }
        RiskProfile::Aggressive => {
            if is_pumpfun { 0.1 } else { 0.05 }
        }
        RiskProfile::Balanced => {
            if is_pumpfun { 0.15 } else { 0.1 }
        }
        RiskProfile::Safe => {
            if is_pumpfun { 0.2 } else { 0.15 }
        }
    }
}

pub fn should_snipe_immediately(profile: RiskProfile) -> bool {
    matches!(profile, RiskProfile::Extreme | RiskProfile::Aggressive)
}

// ========== LIQUIDITY WAITING ==========

pub async fn check_liquidity_enhanced(mint: &str) -> (bool, f64, String) {
    let dexscreener_info = check_dexscreener(mint).await;
    
    if dexscreener_info.is_listed && dexscreener_info.liquidity_usd > 0.0 {
        return (true, dexscreener_info.liquidity_usd, "dexscreener".to_string());
    }
    
    let (has_liquidity, liquidity) = check_liquidity_fast(mint).await;
    if has_liquidity && liquidity > 0.0 {
        return (true, liquidity, "jupiter".to_string());
    }
    
    (false, 0.0, "none".to_string())
}

pub async fn wait_for_liquidity(mint: &str, max_wait_seconds: u64) -> Option<(f64, String)> {
    let start = Instant::now();
    let max_wait = Duration::from_secs(max_wait_seconds);
    
    while start.elapsed() < max_wait {
        let (has_liquidity, liquidity, source) = check_liquidity_enhanced(mint).await;
        
        if has_liquidity && liquidity >= 500.0 {
            return Some((liquidity, source));
        }
        
        if has_liquidity && liquidity > 0.0 {
            // Low liquidity, keep waiting
        }
        
        sleep(Duration::from_millis(1000)).await;
    }
    
    None
}

// ========== PUMP.FUN FILTER ==========

pub fn filter_pump_tokens(
    pairs: &Vec<Value>,
    seen: &mut HashSet<String>,
) -> Vec<Value> {
    let mut valid_tokens = Vec::new();

    for pair in pairs {
        let chain = pair["chainId"].as_str().unwrap_or("");
        let dex = pair["dexId"].as_str().unwrap_or("");

        if chain != "solana" {
            continue;
        }

        if dex != "pumpfun" {
            continue;
        }

        let mint = pair["baseToken"]["address"]
            .as_str()
            .unwrap_or("")
            .to_string();

        if mint.is_empty() || seen.contains(&mint) {
            continue;
        }

        let liquidity = pair["liquidity"]["usd"]
            .as_f64()
            .unwrap_or(0.0);

        if liquidity < 500.0 {
            continue;
        }

        seen.insert(mint);
        valid_tokens.push(pair.clone());
    }

    valid_tokens
}