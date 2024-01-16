use std::io::Write;

use env_logger::Target;
use serde::Serialize;

pub fn init() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .target(Target::Stdout)
        .format(|buf, record| {
            let entry = LogEntry {
                level: record.level().to_string(),
                target: record.target().to_string(),
                message: format!("{}", record.args()),
            };
            let content = serde_json::to_string(&entry).unwrap();
            writeln!(buf, "{}", content)
        })
        .init();
}

#[derive(Serialize)]
struct LogEntry {
    level: String,
    target: String,
    message: String,
}
