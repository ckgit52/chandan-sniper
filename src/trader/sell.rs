use crate::config::Config;
use crate::wallet::Wallet;

pub async fn execute_sell(
    mint: &str,
    amount_percent: f64,
    config: &Config,
    wallet: &Wallet,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("💰 Selling {}% of {}", amount_percent, mint);
    Ok("sell_tx".to_string())
}

pub async fn take_profit(
    mint: &str,
    entry_price: f64,
    target_multiplier: f64,
    config: &Config,
    wallet: &Wallet,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Taking profit at {}x", target_multiplier);
    execute_sell(mint, 100.0, config, wallet).await?;
    Ok(())
}
