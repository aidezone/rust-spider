use std::sync::{Arc, Mutex};
use crate::fetcher::client::HttpClient;
use std::collections::{HashSet, VecDeque};
use std::thread;
use crossbeam::channel::Sender;
use super::TaskInfo;

pub struct Worker {
    http_client: Arc<HttpClient>,
    to_visit: Arc<Mutex<VecDeque<String>>>,
    visited: Arc<Mutex<HashSet<String>>>,
    task_info: Arc<Mutex<TaskInfo>>,
    sender: Sender<TaskInfo>,
}

impl Worker {
    pub fn new(
        http_client: Arc<HttpClient>,
        to_visit: Arc<Mutex<VecDeque<String>>>,
        visited: Arc<Mutex<HashSet<String>>>,
        task_info: Arc<Mutex<TaskInfo>>,
        sender: Sender<TaskInfo>,
    ) -> Self {
        Self {
            http_client,
            to_visit,
            visited,
            task_info,
            sender,
        }
    }

    pub fn run(&self) {
        loop {
            let url = {
                let mut to_visit = self.to_visit.lock().unwrap();
                to_visit.pop_front()
            };

            if let Some(url) = url {
                let mut visited = self.visited.lock().unwrap();
                if visited.contains(&url) {
                    continue;
                }
                visited.insert(url.clone());

                println!("Thread {:?} fetching: {}", thread::current().id(), url);

                // 使用共享的 HttpClient 实例来抓取 URL
                let http_client = self.http_client.clone();
                let links = http_client.get(&url);

                for link in links {
                    let mut to_visit = self.to_visit.lock().unwrap();
                    if !visited.contains(&link) {
                        to_visit.push_back(link);
                    }
                }

                let mut task_info = self.task_info.lock().unwrap();
                task_info.progress += 1;
                self.sender.send(task_info.clone()).unwrap(); // 通知主线程任务进度变化
            } else {
                break;
            }
        }
    }
}
