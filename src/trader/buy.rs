pub fn execute() {
    println!("🟢 Executing BUY...");
    crate::notifier::telegram::send("BUY executed");
}
