use log::LevelFilter;
use simple_logger::SimpleLogger;

pub fn init_logger() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();
}
