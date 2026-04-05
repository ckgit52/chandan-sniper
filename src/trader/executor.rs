use crate::config::Config;
use crate::wallet::Wallet;
use super::{buy::execute_buy, sell::execute_sell};

pub struct TradeExecutor {
    config: Config,
    wallet: Wallet,
}

impl TradeExecutor {
    pub fn new(config: Config, wallet: Wallet) -> Self {
        Self { config, wallet }
    }
    
    pub async fn snipe(&self, mint: &str, quality_score: u8) -> Result<String, Box<dyn std::error::Error>> {
        if quality_score < self.config.min_quality_score {
            println!("❌ Quality score too low: {}/{}", quality_score, self.config.min_quality_score);
            return Err("Quality score too low".into());
        }
        
        let amount = self.config.snipe_amount_sol;
        execute_buy(mint, amount, &self.config, &self.wallet).await
    }
}
