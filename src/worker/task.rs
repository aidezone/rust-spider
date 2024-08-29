use std::collections::{HashSet, VecDeque};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use serde::{Serialize, Deserialize};
use chrono::Utc;

#[derive(Serialize, Deserialize)]
pub struct TaskInfo {
    pub id: u32,
    pub name: String,
    pub progress: u32,
    pub created_at: String,
    pub ended_at: Option<String>,
}

pub enum TaskStatus {
    Running,
    Paused,
    Stopped,
}

pub struct TaskManager {
    visited: HashSet<String>,
    to_visit: VecDeque<String>,
    status: TaskStatus,
    task_info: TaskInfo,
}

impl TaskManager {
    pub fn new(start_urls: Vec<String>, task_name: String) -> Self {
        let task_id = Self::generate_task_id();
        let created_at = Self::current_timestamp();

        let task_info = TaskInfo {
            id: task_id,
            name: task_name.clone(),
            progress: 0,
            created_at,
            ended_at: None,
        };

        let task_dir = format!("tasks/{}", task_name);
        std::fs::create_dir_all(&task_dir).unwrap();

        Self::save_task_info(&task_info, &task_dir);

        TaskManager {
            visited: HashSet::new(),
            to_visit: VecDeque::from(start_urls),
            status: TaskStatus::Running,
            task_info,
        }
    }

    pub fn run(&mut self) {
        self.status = TaskStatus::Running;
        while let Some(url) = self.to_visit.pop_front() {
            if let TaskStatus::Paused | TaskStatus::Stopped = self.status {
                break;
            }

            if !self.visited.contains(&url) {
                self.visited.insert(url.clone());
                println!("Fetching: {}", url);

                let links = self.fetch(&url);

                for link in links {
                    if !self.visited.contains(&link) {
                        self.to_visit.push_back(link);
                    }
                }

                self.task_info.progress += 1;
                Self::save_task_info(&self.task_info, &self.get_task_dir());
            }
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
}
