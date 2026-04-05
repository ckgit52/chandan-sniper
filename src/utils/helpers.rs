use std::time::{SystemTime, UNIX_EPOCH};

pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn format_sol(lamports: u64) -> String {
    format!("{:.6}", lamports as f64 / 1_000_000_000.0)
}

pub fn truncate_mint(mint: &str) -> String {
    if mint.len() > 12 {
        format!("{}...{}", &mint[..6], &mint[mint.len()-6..])
    } else {
        mint.to_string()
    }
}
