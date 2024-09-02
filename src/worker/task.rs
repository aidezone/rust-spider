use std::collections::{HashSet, VecDeque};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};
use std::fmt;
use crossbeam::channel;
use serde::{Serialize, Deserialize};
use chrono::Utc;
use super::Worker;
use crate::fetcher::client::HttpClient;
use prettytable::{cell, row, Table};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
// 引入宏
use crate::info;
use crate::warn;
use crate::error;
use crate::TERMINATE;
// use crate::utils::logging;
// use crate::utils::logging::{info, warn, error};

#[derive(Serialize, Deserialize, Clone)]
pub struct TaskInfo {
    pub id: u32,
    pub name: String,
    pub progress: u32,
    pub deep: u32,
    pub status: TaskStatus,
    pub created_at: Option<String>,
    pub ended_at: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum TaskStatus {
    Idle,
    Running,
    Finished,
    Stopped,
}

// 为 TaskStatus 实现 Display trait
impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = match self {
            TaskStatus::Idle => "Idle",
            TaskStatus::Running => "Running",
            TaskStatus::Finished => "Finished",
            TaskStatus::Stopped => "Stopped",
        };
        write!(f, "{}", status)
    }
}

#[derive(Clone)]
pub struct TaskManager {
    visited: HashSet<String>,
    to_visit: VecDeque<String>,
    task_info: TaskInfo,
    task_dir: String,
}

impl TaskManager {
    pub fn new() -> Self {
        TaskManager {
            visited: HashSet::new(),
            to_visit: VecDeque::new(),
            task_dir: String::new(),
            task_info: TaskInfo {
                id: 0,
                name: String::new(),
                deep: 0,
                progress: 0,
                created_at: None,
                ended_at: None,
                status: TaskStatus::Idle,
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
        self.task_dir = format!("tasks/{}", self.task_info.name);

        std::fs::create_dir_all(&self.task_dir).expect("Failed to create task directory");

        // Save task information to a file
        Self::save_task_info(&self.task_info, &self.task_dir);
    }

    pub fn run(&mut self, m: &Arc<MultiProgress>) {

        self.task_info.status = TaskStatus::Running;

        let num_threads = num_cpus::get();
        let (sender, receiver) = channel::unbounded();

        // 计算每个 worker 应处理的 URL 数量
        let chunk_size = (self.to_visit.len() + num_threads - 1) / num_threads;

        let visited = Arc::new(Mutex::new(self.visited.clone()));
        let task_info = Arc::new(Mutex::new(self.task_info.clone()));
        let http_client = Arc::new(HttpClient::new(None));

        // 创建一个新的进度条，长度根据任务总数调整
        // let pb = ProgressBar::new(self.to_visit.len() as u64);
        let pb = m.add(ProgressBar::new(self.to_visit.len() as u64));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
                .unwrap()  // 确保 template 调用成功，否则会导致 panic
                .progress_chars("#>-"),
        );

        // 将 VecDeque 转换为 Vec
        // let to_visit_vec: Vec<_> = self.to_visit.lock().unwrap().clone().into_iter().collect();
        let to_visit_vec: Vec<_> = self.to_visit.clone().into_iter().collect();

        // 创建 worker 线程
        let mut handles = Vec::new();
        for chunk in to_visit_vec.chunks(chunk_size) {
            let worker_to_visit = VecDeque::from(chunk.to_vec()); // 将 chunk 转换为 VecDeque

            let worker = Worker::new(
                Arc::clone(&http_client),
                Arc::new(Mutex::new(worker_to_visit)),
                Arc::clone(&visited),
                Arc::clone(&task_info),
                sender.clone(),
            );

            let handle = thread::spawn(move || {
                worker.run();
            });

            handles.push(handle);
        }
        info!("Thread Number {:?}", handles.len());

        // 主线程处理任务保存和状态检查
        while self.task_info.status == TaskStatus::Running {
            // 从 receiver 接收更新的任务信息
            if let Ok(updated_task_info) = receiver.recv() {
                self.task_info.progress = updated_task_info.progress;
                // 更新进度条
                // pb.inc(1);
                pb.set_position(self.task_info.progress as u64);
                // pb.set_position((self.task_info.progress / self.to_visit.len() * 100).into());
                pb.set_message(format!("Processing item {}", self.task_info.progress));
                Self::save_task_info(&self.task_info, &self.get_task_dir());
                if self.task_info.progress == (self.to_visit.len() as u32) {
                    break;
                }
            }

            // 处理终止信号
            if TERMINATE.load(Ordering::SeqCst) {
                self.task_info.status = TaskStatus::Stopped;
                break;
            }
        }

        // 等待所有 worker 完成
        for handle in handles.drain(..) { // 确保迭代完成后清空 handles
            info!("Wait for thread exiting...");
            handle.join().unwrap();
        }

        if self.task_info.status == TaskStatus::Running {
            self.task_info.status = TaskStatus::Finished;
            pb.finish_with_message("Done!"); // 完成并显示结束消息
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
        table.add_row(row!["Task ID", "Task Name", "Deep", "Progress", "Status", "Created At", "Ended At"]);

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
                            task_info.status,
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
