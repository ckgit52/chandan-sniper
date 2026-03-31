pub fn apply(liquidity: f64) {
    if liquidity > 8000.0 {
        println!("✅ Passed filters");
        crate::trader::buy::execute();
    } else {
        println!("❌ Skipped");
    }
}
