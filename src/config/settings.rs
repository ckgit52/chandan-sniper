use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub helius_api_key: String,
    pub rpc_url: String,
    pub private_key: String,
    pub snipe_amount_sol: f64,
    pub max_slippage_bps: u16,
    pub priority_fee_sol: f64,
    pub min_liquidity_usd: f64,
    pub min_quality_score: u8,
    pub telegram_bot_token: Option<String>,
    pub telegram_chat_id: Option<String>,
}

impl Config {
    pub fn load() -> Self {
        dotenv::dotenv().ok();
        
        Self {
            helius_api_key: env::var("HELIUS_API_KEY").unwrap_or_else(|_| "276d33c2-1fc1-48e0-9d1a-6ecbf4d31ab5".to_string()),
            rpc_url: env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
            private_key: env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set in .env"),
            snipe_amount_sol: env::var("SNIPE_AMOUNT_SOL").unwrap_or_else(|_| "0.1".to_string()).parse().unwrap(),
            max_slippage_bps: env::var("MAX_SLIPPAGE_BPS").unwrap_or_else(|_| "1500".to_string()).parse().unwrap(),
            priority_fee_sol: env::var("PRIORITY_FEE_SOL").unwrap_or_else(|_| "0.015".to_string()).parse().unwrap(),
            min_liquidity_usd: env::var("MIN_LIQUIDITY_USD").unwrap_or_else(|_| "500".to_string()).parse().unwrap(),
            min_quality_score: env::var("MIN_QUALITY_SCORE").unwrap_or_else(|_| "60".to_string()).parse().unwrap(),
            telegram_bot_token: env::var("TELEGRAM_BOT_TOKEN").ok(),
            telegram_chat_id: env::var("TELEGRAM_CHAT_ID").ok(),
        }
    }
}
