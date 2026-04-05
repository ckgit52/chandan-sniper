use reqwest::Client;
use serde_json::json;
use std::env;
use dotenv::dotenv;

pub async fn send_telegram_alert(
    bot_token: &str,
    chat_id: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);
    
    let body = json!({
        "chat_id": chat_id,
        "text": message,
        "parse_mode": "HTML",
        "disable_web_page_preview": true,
    });
    
    let response = client.post(&url).json(&body).send().await?;
    
    if response.status().is_success() {
        println!("📱 Telegram alert sent!");
    } else {
        let error_text = response.text().await?;
        println!("⚠️ Telegram error: {}", error_text);
    }
    
    Ok(())
}

pub async fn send_snipe_alert(mint: &str, quality: u8, tx_link: &str, amount_sol: f64) {
    dotenv().ok();
    
    let bot_token = env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default();
    let chat_id = env::var("TELEGRAM_CHAT_ID").unwrap_or_default();
    
    if bot_token.is_empty() || chat_id.is_empty() {
        println!("⚠️ Telegram not configured - missing BOT_TOKEN or CHAT_ID");
        return;
    }
    
    let message = format!(
        "🎯 <b>🔥 NEW TOKEN SNIPED!</b>\n\n━━━━━━━━━━━━━━━━━━━━━━\n🪙 <b>Token</b>: <code>{}</code>\n📊 <b>Quality Score</b>: {}/100\n💰 <b>Amount</b>: {} SOL\n🔗 <b>View</b>: <a href='{}'>Solscan</a>\n━━━━━━━━━━━━━━━━━━━━━━\n⚡️ <i>Snipe executed!</i>",
        mint, quality, amount_sol, tx_link
    );
    
    let _ = send_telegram_alert(&bot_token, &chat_id, &message).await;
}
