use std::fs::OpenOptions;
use std::io::Write;
use std::env;
use std::process::Command;

pub fn log_error(message: &str) {
    let log_file_path = "error.log";
    let mut file = OpenOptions::new().append(true).create(true).open(log_file_path).unwrap_or_else(|_| {
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
    let mut file = OpenOptions::new().append(true).create(true).open(log_file_path).unwrap_or_else(|_| {
        eprintln!("Could not create log file.");
        std::process::exit(1);
    });
    writeln!(file, "INFO: {}", message).unwrap_or_else(|_| {
        eprintln!("Could not write to log file.");
        std::process::exit(1);
    });
}

pub fn get_hostname() -> String {
    env::var("COMPUTERNAME").unwrap_or_else(|_| {
        Command::new("hostname")
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    })
}