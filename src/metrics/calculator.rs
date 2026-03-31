pub fn calculate(liquidity: f64) {
    println!("📊 Liquidity: {}", liquidity);
    crate::filters::basic_filters::apply(liquidity);
}
