use::chrono;

use crate::file_manager;

#[derive(Debug)]
enum EntryType {
    Error,
    Action,
    FileOperation,
    ServerConection,
}



struct Logger {
    entry_type: EntryType,
    timestamp: String,
    message: String,
}

impl Logger {
    pub fn new(entry_type: EntryType, message: String) -> Self {
        let now = chrono::Local::now();
        let timestamp = now.format("%d-%m-%Y %H:%M:%S").to_string();
        Logger {
            entry_type,
            timestamp,
            message,
        }
    }


    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let entry = format!(
            "{{\"type\": \"{:?}\",\"timestamp\": \"{}\",\"message\": \"{}\"}}",
            self.entry_type, self.timestamp, self.message
        );
        file_manager::append_to_file("src/log.json".to_string(), entry)?;
        Ok(())
    }

}


pub fn log_error(message: String) -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::new(EntryType::Error, message);
    logger.save()?;
    Ok(())
}

pub fn log_action(message: String) -> Result<(), Box<dyn std::error::Error>> {
    let logger = Logger::new(EntryType::Action, message);
    logger.save()?;
    Ok(())
}

pub fn log (flags: Vec<String>) -> Result<(), Box<dyn std::error::Error>>{
    log_error(String::from("error mesaage"))?;
    Ok(())
}