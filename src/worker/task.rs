use std::collections::{HashSet, VecDeque};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use crossbeam::channel;
use serde::{Serialize, Deserialize};
use chrono::Utc;
use super::Worker;
use crate::fetcher::client::HttpClient;
use prettytable::{cell, row, Table};

#[derive(Serialize, Deserialize, Clone)]
pub struct TaskInfo {
    pub id: u32,
    pub name: String,
    pub progress: u32,
    pub deep: u32,
    pub created_at: Option<String>,
    pub ended_at: Option<String>,
}

#[derive(PartialEq)]
pub enum TaskStatus {
    Idle,
    Running,
    Paused,
    Stopped,
}

pub struct TaskManager {
    visited: HashSet<String>,
    to_visit: VecDeque<String>,
    status: TaskStatus,
    task_info: TaskInfo,
    task_dir: String,
}

impl TaskManager {
    pub fn new() -> Self {
        TaskManager {
            visited: HashSet::new(),
            to_visit: VecDeque::new(),
            status: TaskStatus::Idle,
            task_dir: String::new(),
            task_info: TaskInfo {
                id: 0,
                name: String::new(),
                deep: 0,
                progress: 0,
                created_at: None,
                ended_at: None,
            },
        }
    }

    // New method to handle setting up and saving a task
    pub fn create_task(&mut self, start_urls: Vec<String>, task_name: String, deep: u32) {
        self.task_info.id = Self::generate_task_id();
        self.task_info.name = task_name.clone();
        self.task_info.deep = deep;
        self.task_info.created_at = Some(Self::current_timestamp());
        self.to_visit = VecDeque::from(start_urls);
        self.status = TaskStatus::Running;
        self.task_dir = format!("tasks/{}", self.task_info.name);

        std::fs::create_dir_all(&self.task_dir).expect("Failed to create task directory");

        // Save task information to a file
        Self::save_task_info(&self.task_info, &self.task_dir);
    }

    pub fn run(&mut self) {
        self.status = TaskStatus::Running;

        let num_threads = num_cpus::get();
        let (sender, receiver) = channel::unbounded();

        let to_visit = Arc::new(Mutex::new(self.to_visit.clone()));
        let visited = Arc::new(Mutex::new(self.visited.clone()));
        let task_info = Arc::new(Mutex::new(self.task_info.clone()));
        let http_client = Arc::new(HttpClient::new(None));

        // 创建 worker 线程
        let mut handles = Vec::new();
        for _ in 0..num_threads {
            let worker = Worker::new(
                Arc::clone(&http_client),
                Arc::clone(&to_visit),
                Arc::clone(&visited),
                Arc::clone(&task_info),
                sender.clone(),
            );

            let handle = thread::spawn(move || {
                worker.run();
            });

            handles.push(handle);
        }

        // 主线程处理任务保存和状态检查
        while self.status == TaskStatus::Running {
            if let Ok(updated_task_info) = receiver.recv() {
                self.task_info = updated_task_info;
                Self::save_task_info(&self.task_info, &self.get_task_dir());
            }
        }

        // 等待所有 worker 完成
        for handle in handles {
            handle.join().unwrap();
        }

        self.task_info.ended_at = Some(Self::current_timestamp());
        Self::save_task_info(&self.task_info, &self.get_task_dir());
    }

    fn fetch(&self, url: &str) -> Vec<String> {
        vec![format!("{}/link1", url), format!("{}/link2", url)]
    }

    pub fn save_state(&self, file_path: &str) -> std::io::Result<()> {
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(file_path)?;
        for url in &self.visited {
            writeln!(file, "{}", url)?;
        }
        Ok(())
    }

    pub fn load_state(&mut self, file_path: &str) -> std::io::Result<()> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            if let Ok(url) = line {
                self.visited.insert(url);
            }
        }
        Ok(())
    }

    pub fn pause(&mut self) {
        self.status = TaskStatus::Paused;
    }

    pub fn stop(&mut self) {
        self.status = TaskStatus::Stopped;
    }

    fn generate_task_id() -> u32 {
        Utc::now().timestamp() as u32
    }

    fn current_timestamp() -> String {
        Utc::now().to_rfc3339()
    }

    fn get_task_dir(&self) -> String {
        format!("tasks/{}", self.task_info.name)
    }

    fn save_task_info(task_info: &TaskInfo, task_dir: &str) {
        let task_info_path = format!("{}/task_info.json", task_dir);
        let file = File::create(task_info_path).unwrap();
        serde_json::to_writer(file, task_info).unwrap();
    }

    // Helper function to read a task's information from its file
    fn read_task_info(task_dir: &Path) -> Option<TaskInfo> {
        let task_file = task_dir.join("task_info.json");
        if task_file.exists() {
            let file = File::open(task_file).ok()?;
            let reader = BufReader::new(file);
            let task_info: TaskInfo = serde_json::from_reader(reader).ok()?;
            Some(task_info)
        } else {
            None
        }
    }

    // Function to list tasks
    pub fn list_tasks(&self) {
        let mut table = Table::new();
        table.add_row(row!["Task ID", "Task Name", "Deep", "Progress", "Created At", "Ended At"]);

        // Read tasks from the directory
        if let Ok(entries) = fs::read_dir("tasks") {
            for entry in entries.flatten() {
                if entry.file_type().map(|f| f.is_dir()).unwrap_or(false) {
                    let task_dir = entry.path();
                    if let Some(task_info) = TaskManager::read_task_info(&task_dir) {
                        table.add_row(row![
                            task_info.id,
                            task_info.name,
                            task_info.deep,
                            task_info.progress,
                            task_info.created_at.unwrap_or_else(|| "N/A".to_string()),
                            task_info.ended_at.unwrap_or_else(|| "N/A".to_string())
                        ]);
                    }
                }
            }
        }

        table.printstd();
    }
}
