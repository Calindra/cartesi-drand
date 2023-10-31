use log::{set_boxed_logger, Level, Metadata, Record, SetLoggerError};

pub struct SimpleLogger {
    namespace: &'static str,
}

impl SimpleLogger {
    pub fn new(namespace: &'static str) -> Self {
        Self { namespace }
    }
    fn get_record_str(&self, record: &Record) -> String {
        format!(
            "{} {}: {}:{:?} {}",
            self.namespace,
            record.level(),
            record.target(),
            record.line(),
            record.args(),
        )
    }
    pub fn init(self) -> Result<(), SetLoggerError> {
        self.init_max_level(log::LevelFilter::Info)
    }
    pub fn init_max_level(self, level: log::LevelFilter) -> Result<(), SetLoggerError> {
        let logger = Box::new(self);
        set_boxed_logger(logger).map(|()| log::set_max_level(level))
    }
}

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        let level = record.level();
        match level {
            Level::Error => {
                let record_str = self.get_record_str(record);
                eprintln!("{}", record_str);
            }
            Level::Debug => {
                let record_str = self.get_record_str(record);
                println!("{}", record_str);
            }
            _ => {
                println!("{} {}: {}", self.namespace, record.level(), record.args());
            }
        }
    }

    fn flush(&self) {}
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_works() {
        let logger = SimpleLogger { namespace: "TEST" };
        let result = logger.init();
        assert!(result.is_ok());
    }
}
