use std::fs;
use std::process::Command;
use crate::worker::{TaskManager, TaskStatus};
use crate::utils::parse_args;

mod config;
mod worker;
mod fetcher;
mod parser;
mod writer;
mod utils;
mod aspect;

fn main() {
    let (matches, mut app) = parse_args();

    let mut task_manager = TaskManager::new();

    match matches.subcommand() {
        Some(("create", sub_m)) => {
            let file = sub_m.get_one::<String>("file").unwrap();
            let name = sub_m.get_one::<String>("name").unwrap();
            let deep = sub_m.get_one::<u32>("deep").unwrap();

            let start_urls = fs::read_to_string(file).expect("Could not read file")
                .lines()
                .map(|line| line.to_string())
                .collect();
            println!("Creating and starting new task: {}", name);
            task_manager.create_task(start_urls, name.to_string(), *deep);
            task_manager.run();
            task_manager.save_state(&format!("tasks/{}/visited_urls.txt", name)).unwrap();
        }
        Some(("list", _sub_m)) => {
            println!("Listing all tasks...");
            task_manager.list_tasks();  // Call the list_tasks method
        }
        Some(("resume", sub_m)) => {
            let task_id = sub_m.get_one::<String>("id").unwrap();
            println!("Resuming task with ID: {}", task_id);
            // Implement task resume logic here
        }
        Some(("restart", sub_m)) => {
            let task_id = sub_m.get_one::<String>("id").unwrap();
            println!("Restarting task with ID: {}", task_id);
            // Implement task restart logic here
        }
        _ => {
            let _ = app.print_help().unwrap();
            println!();
        }
        // _ => unreachable!("Clap should ensure only valid subcommands are used"),
    }
}
