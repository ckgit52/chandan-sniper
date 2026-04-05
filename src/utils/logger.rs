use std::fs::File;
use std::io::Write;
use std::sync::Mutex;

pub struct Logger {
    file: Mutex<File>,
}

impl Logger {
    pub fn new(path: &str) -> Self {
        let file = File::create(path).unwrap();
        Self { file: Mutex::new(file) }
    }
    
    pub fn log(&self, message: &str) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_line = format!("[{}] {}\n", timestamp, message);
        
        println!("{}", log_line.trim());
        
        if let Ok(mut file) = self.file.lock() {
            let _ = file.write_all(log_line.as_bytes());
            let _ = file.flush();
        }
    }
}
