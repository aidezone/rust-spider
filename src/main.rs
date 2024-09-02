
#[macro_use]
extern crate lazy_static;

use std::fs;
use std::process::Command;
use std::process::exit;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use crate::worker::{TaskManager, TaskStatus};
use crate::utils::parse_args;
use crate::utils::logging::Logging;
use ctrlc;

static TERMINATE: AtomicBool = AtomicBool::new(false);

lazy_static! {
    pub static ref MULTI_PROGRESS: Arc<MultiProgress> = Arc::new(MultiProgress::new());
}


mod config;
mod worker;
mod fetcher;
mod parser;
mod writer;
mod utils;

fn main() {

    // 初始化 logger
    let _logger = Logging::new(&MULTI_PROGRESS);

    let m = MULTI_PROGRESS.clone();

    let (matches, mut app) = parse_args();

    // let task_manager = Arc::new(Mutex::new(TaskManager::new()));
    let mut task_manager = TaskManager::new();

    // 设置 Ctrl+C 处理
    ctrlc::set_handler(move || {
        println!("Task state saved. Exiting...");
        TERMINATE.store(true, Ordering::SeqCst);
    }).expect("Error setting Ctrl+C handler");

    match matches.subcommand() {
        Some(("create", sub_m)) => {
            let file = sub_m.get_one::<String>("file").unwrap();
            let name = sub_m.get_one::<String>("name").unwrap();
            let deep = sub_m.get_one::<u32>("deep").unwrap();
            // let mut task_manager = task_manager.lock().unwrap();
            let start_urls = fs::read_to_string(file).expect("Could not read file")
                .lines()
                .map(|line| line.to_string())
                .collect();
            println!("Creating and starting new task: {}", name);
            task_manager.create_task(start_urls, name.to_string(), *deep);
            task_manager.run(&m);
            println!("All threads finished!");
            task_manager.save_state(&format!("tasks/{}/visited_urls.txt", name)).unwrap();
        }
        Some(("list", _sub_m)) => {
            // println!("Listing all tasks...");
            // let mut task_manager = task_manager.lock().unwrap();
            task_manager.list_tasks();  // Call the list_tasks method
        }
        Some(("resume", sub_m)) => {
            let task_id = sub_m.get_one::<u32>("id").unwrap(); // task_id is of type &u32
            println!("Resuming task with ID: {}", *task_id);   // Dereference task_id to get u32、
            // Implement task resume logic here
        }
        Some(("restart", sub_m)) => {
            let task_id = sub_m.get_one::<u32>("id").unwrap();
            println!("Restarting task with ID: {}", *task_id);
            // Implement task restart logic here
        }
        _ => {
            let _ = app.print_help().unwrap();
            println!();
        }
        // _ => unreachable!("Clap should ensure only valid subcommands are used"),
    }
}
