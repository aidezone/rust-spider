use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use indicatif::{ProgressBar, MultiProgress};
use indicatif::ProgressStyle;
use crate::MULTI_PROGRESS;
use std::collections::VecDeque;
use crate::TERMINATE;

const MAX_LOG_LINES: usize = 20; // 定长队列的最大行数

pub struct Logging {
    log_lines: Arc<Mutex<VecDeque<String>>>,
    tx: Sender<String>,
    pb: ProgressBar,
}

impl Logging {
    pub fn new(m: &MultiProgress) -> Self {
        let (tx, rx) = channel();
        let log_lines = Arc::new(Mutex::new(VecDeque::with_capacity(MAX_LOG_LINES)));
        // 创建一个用于显示日志信息的进度条
        let pb = m.add(ProgressBar::new_spinner());
        pb.set_style(ProgressStyle::default_spinner().template("{msg}").unwrap());

        let log_lines_clone = Arc::clone(&log_lines);
        let mut pb_clone = pb.clone();
        // 使用 Arc<Mutex<...>> 来共享 log_lines
        thread::spawn(move || {
            // while !TERMINATE.load(Ordering::SeqCst) {
                while let Ok(message) = rx.recv() {
                    let mut log_lines = log_lines_clone.lock().unwrap();
                    if log_lines.len() == MAX_LOG_LINES {
                        log_lines.pop_front();
                    }
                    log_lines.push_back(message);
                    
                    // 将最新的日志内容拼接成一个大字符串
                    let log_message = log_lines.iter().cloned().collect::<Vec<_>>().join("\n");
                    pb_clone.set_message(log_message);
                }
            // }
        });

        Logging { log_lines, tx, pb }
    }

    pub fn log(&self, level: &str, message: String) {
        self.tx.send(format!("{}: {}", level, message)).unwrap();
    }
}

lazy_static! {
    pub static ref LOGGER: Arc<Mutex<Logging>> = Arc::new(Mutex::new(Logging::new(&MULTI_PROGRESS)));
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        if let Ok(logger) = $crate::utils::logging::LOGGER.lock() {
            logger.log("INFO", format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        if let Ok(logger) = $crate::utils::logging::LOGGER.lock() {
            logger.log("WARN", format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        if let Ok(logger) = $crate::utils::logging::LOGGER.lock() {
            logger.log("ERROR", format!($($arg)*));
        }
    };
}
