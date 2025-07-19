
use std::process::Command;
use std::env;
use crate::config::Config;

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn from_string(s: &str) -> LogLevel {
        match s.to_lowercase().as_str() {
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" => LogLevel::Warn,
            "error" => LogLevel::Error,
            _ => LogLevel::Info, // Default fallback
        }
    }
    
    pub fn should_log(&self, target_level: LogLevel) -> bool {
        match self {
            LogLevel::Debug => true, // Debug shows everything
            LogLevel::Info => matches!(target_level, LogLevel::Info | LogLevel::Warn | LogLevel::Error),
            LogLevel::Warn => matches!(target_level, LogLevel::Warn | LogLevel::Error),
            LogLevel::Error => matches!(target_level, LogLevel::Error),
        }
    }
}

pub struct Logger {
    global_level: LogLevel,
    mqtt_debug: bool,
    influxdb_debug: bool,
    api_debug: bool,
}

impl Logger {
    pub fn new(config: &Config) -> Self {
        Self {
            global_level: LogLevel::from_string(&config.log_level),
            mqtt_debug: config.enable_mqtt_debug,
            influxdb_debug: config.enable_influxdb_debug,
            api_debug: config.enable_api_debug,
        }
    }
    
    pub fn debug(&self, module: &str, message: &str) {
        let should_show = match module {
            "mqtt" => self.mqtt_debug,
            "influxdb" => self.influxdb_debug,
            "api" => self.api_debug,
            _ => self.global_level.should_log(LogLevel::Debug),
        };
        
        if should_show {
            println!("🔍 [DEBUG][{}] {}", module.to_uppercase(), message);
        }
    }
    
    pub fn info(&self, module: &str, message: &str) {
        let should_show = match module {
            "mqtt" => self.mqtt_debug || self.global_level.should_log(LogLevel::Info),
            "influxdb" => self.influxdb_debug || self.global_level.should_log(LogLevel::Info),
            "api" => self.api_debug || self.global_level.should_log(LogLevel::Info),
            _ => self.global_level.should_log(LogLevel::Info),
        };
        
        if should_show {
            println!("ℹ️  [INFO][{}] {}", module.to_uppercase(), message);
        }
    }
    
    pub fn warn(&self, module: &str, message: &str) {
        // Warnings always show regardless of debug settings
        println!("⚠️  [WARN][{}] {}", module.to_uppercase(), message);
    }
    
    pub fn error(&self, module: &str, message: &str) {
        // Errors always show regardless of debug settings
        println!("❌ [ERROR][{}] {}", module.to_uppercase(), message);
    }
}

pub fn get_hostname() -> String {
    env::var("COMPUTERNAME").unwrap_or_else(|_| {
        Command::new("hostname")
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    })
}
