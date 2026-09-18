//! HAAVK 日志系统：控制台输出 + 文件落盘（`~/.haavk/logs/haavk.log`）

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::sync::{Mutex, OnceLock};

use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};

use crate::haavk_home;

/// 线程安全日志写入器
pub struct HaavkLogger {
    level: LevelFilter,
    file: Option<Mutex<fs::File>>,
}

static LOGGER: OnceLock<HaavkLogger> = OnceLock::new();

impl HaavkLogger {
    /// 初始化日志。`level` 取值：trace/debug/info/warn/error
    pub fn init(level: &str) -> Result<(), SetLoggerError> {
        let level = match level.to_ascii_lowercase().as_str() {
            "trace" => LevelFilter::Trace,
            "debug" => LevelFilter::Debug,
            "warn" => LevelFilter::Warn,
            "error" => LevelFilter::Error,
            _ => LevelFilter::Info,
        };

        // 日志目录：$HAAVK_HOME/logs
        let log_dir = haavk_home().join("logs");
        let _ = fs::create_dir_all(&log_dir);
        let log_path = log_dir.join("haavk.log");
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .ok()
            .map(Mutex::new);

        let logger = HaavkLogger { level, file };
        let logger = LOGGER.get_or_init(|| logger);
        let _ = log::set_logger(logger);
        log::set_max_level(level);
        Ok(())
    }
}

impl Log for HaavkLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let line = format!("[{}][{}] {}", ts, record.level(), record.args());

        // 控制台
        if record.level() <= log::Level::Warn {
            eprintln!("{line}");
        } else {
            println!("{line}");
        }

        // 文件
        if let Some(f) = &self.file {
            if let Ok(mut f) = f.lock() {
                let _ = writeln!(f, "{line}");
            }
        }
    }

    fn flush(&self) {
        if let Some(f) = &self.file {
            if let Ok(mut f) = f.lock() {
                let _ = f.flush();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_filter_works() {
        assert!(LevelFilter::Info <= LevelFilter::Trace);
        assert!(LevelFilter::Error <= LevelFilter::Warn);
    }
}
