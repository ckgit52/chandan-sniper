use crate::config::Config;
use crate::wallet::Wallet;
use reqwest::Client;
use serde_json::{json, Value};

pub async fn execute_buy(
    mint: &str,
    amount_sol: f64,
    config: &Config,
    wallet: &Wallet,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("💰 Buying {} SOL of {}", amount_sol, mint);
    
    let quote = get_jupiter_quote(mint, amount_sol, config).await?;
    let tx = execute_jupiter_swap(quote, wallet, config).await?;
    
    Ok(tx)
}

async fn get_jupiter_quote(
    mint: &str,
    amount_sol: f64,
    config: &Config,
) -> Result<Value, Box<dyn std::error::Error>> {
    let client = Client::new();
    let amount_lamports = (amount_sol * 1_000_000_000.0) as u64;
    
    let url = format!(
        "https://quote-api.jup.ag/v6/quote?inputMint=So11111111111111111111111111111111111111112&outputMint={}&amount={}&slippageBps={}",
        mint, amount_lamports, config.max_slippage_bps
    );
    
    let response = client.get(&url).send().await?;
    let quote = response.json::<Value>().await?;
    
    Ok(quote)
}

async fn execute_jupiter_swap(
    quote: Value,
    wallet: &Wallet,
    config: &Config,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    
    let swap_body = json!({
        "quoteResponse": quote,
        "userPublicKey": wallet.pubkey_string(),
        "wrapAndUnwrapSol": true,
        "dynamicComputeUnitLimit": true,
        "prioritizationFeeLamports": (config.priority_fee_sol * 1_000_000_000.0) as u64,
    });
    
    let response = client
        .post("https://quote-api.jup.ag/v6/swap")
        .json(&swap_body)
        .send()
        .await?;
    
    let swap_data = response.json::<Value>().await?;
    let tx_data = swap_data["swapTransaction"].as_str().unwrap();
    
    println!("✅ Buy transaction prepared: {}", &tx_data[..64]);
    
    Ok(tx_data.to_string())
}
