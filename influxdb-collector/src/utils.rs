/*
use std::fs::File;
use std::io::Write;

pub fn log_error(message: &str) {
    let log_file_path = "error.log";
    let mut file = File::create(log_file_path).unwrap_or_else(|_| {
        eprintln!("Could not create log file.");
        std::process::exit(1);
    });
    writeln!(file, "ERROR: {}", message).unwrap_or_else(|_| {
        eprintln!("Could not write to log file.");
        std::process::exit(1);
    });
}

pub fn log_info(message: &str) {
    let log_file_path = "info.log";
    let mut file = File::create(log_file_path).unwrap_or_else(|_| {
        eprintln!("Could not create log file.");
        std::process::exit(1);
    });
    writeln!(file, "INFO: {}", message).unwrap_or_else(|_| {
        eprintln!("Could not write to log file.");
        std::process::exit(1);
    });
}
*/