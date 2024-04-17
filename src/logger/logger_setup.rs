use std::io::Write;
use std::str::FromStr;

use log::{Level, LevelFilter};

const RESET_COLOR_TOKEN: &str = "\x1B[0m";

pub fn setup_logger(verbose: bool) {
    let level_filter = match dotenvy::dotenv() {
        Ok(_) => LevelFilter::from_str(&std::env::var("RUST_LOG").unwrap()).unwrap(),
        Err(_) => match verbose {
            true => LevelFilter::Debug,
            false => log::LevelFilter::Info,
        },
    };
    let mut builder = env_logger::builder();
    builder.filter_level(level_filter);
    builder.format(move |buf, record| {
        let level = record.level();
        let file = record.file().unwrap_or("");
        let color = match level {
            Level::Info => "\x1B[32m",  // Green
            Level::Warn => "\x1B[33m",  // Yellow
            Level::Error => "\x1B[31m", // Red
            Level::Debug => "\x1B[34m", // Blue
            Level::Trace => "\x1B[35m", // Magenta
        };

        if verbose {
            writeln!(
                buf,
                "[{date_time} {color}{level}{RESET_COLOR_TOKEN} {file}] {color}{args}{RESET_COLOR_TOKEN}", // The \x1B[0m sequence resets the color
                date_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                args = record.args()
            )
        } else {
            writeln!(
                buf,
                "{color}{args}{RESET_COLOR_TOKEN}", // The \x1B[0m sequence resets the color
                args = record.args()
            )
        }
    });
    builder.init();
}