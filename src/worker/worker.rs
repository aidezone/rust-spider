use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use crate::fetcher::client::HttpClient;
use crate::parser::html_parser::Parser;
use std::collections::{HashSet, VecDeque};
use std::thread;
use crossbeam::channel::Sender;
use super::{TaskInfo, TaskStatus};
// 引入宏
use crate::info;
use crate::warn;
use crate::error;
use crate::TERMINATE;

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
        let parser = Parser::new();
        while !TERMINATE.load(Ordering::SeqCst) {
            // 从 to_visit 队列中取出一个 URL
            let url = {
                let mut to_visit = self.to_visit.lock().unwrap();
                to_visit.pop_front()
            };

            // 如果有 URL，调用 crawl 方法进行递归抓取
            if let Some(url) = url {
                self.crawl(url, 1, parser.clone());

                // 更新任务进度
                let mut task_info = self.task_info.lock().unwrap();
                task_info.progress += 1;
                self.sender.send(task_info.clone()).unwrap(); // 通知主线程任务进度变化
            } else {
                // 队列为空，结束抓取
                break;
            }
        }
        info!("Thread {:?} finished!", thread::current().id());
    }

    // 递归抓取方法，接收一个 URL 和当前深度
    fn crawl(&self, url: String, current_depth: u32, parser: Parser) {
        if TERMINATE.load(Ordering::SeqCst) {
            return;
        }

        if self.task_info.lock().unwrap().status != TaskStatus::Running {
            return;
        }

        // 检查是否超过最大抓取深度
        if current_depth > self.task_info.lock().unwrap().deep {
            return;
        }

        {
            // 检查 URL 是否已经访问过
            let mut visited = self.visited.lock().unwrap();
            if visited.contains(&url) {
                return;
            }
            visited.insert(url.clone());
        }

        info!("Thread {:?} fetching: {}", thread::current().id(), url);

        // 使用共享的 HttpClient 实例抓取 URL
        let http_client = Arc::clone(&self.http_client);

        match http_client.get(&url) {
            Ok(content) => {
                // 解析抓取到的页面内容
                let parsed_url = reqwest::Url::parse(&url).unwrap();
                let Ok((links, is_content_page, title, text_content)) = parser.parse_content(&content, &parsed_url) else { 
                    error!("Failed to parse content for URL {}", url);
                    return;
                };

                if is_content_page {
                    if let Some(title) = title {
                        info!("Title: {}", title);
                    }
                    if let Some(text_content) = text_content {
                        info!("Content-Length: {}", text_content.len());
                    }
                }

                info!("Links-Count: {}", links.len());

                // 递归抓取所有子链接
                for link in links {
                    if TERMINATE.load(Ordering::SeqCst) {
                        break;
                    }
                    let absolute_url = if let Ok(absolute_url) = parsed_url.join(&link) {
                        absolute_url
                    } else {
                        error!("Failed to resolve link {} relative to base URL {}", link, url);
                        continue;
                    };

                    {
                        // 确保对 visited 的修改是线程安全的
                        let mut visited = self.visited.lock().unwrap();
                        if !visited.contains(absolute_url.as_str()) {
                            drop(visited); // 释放锁以避免死锁
                            self.crawl(absolute_url.to_string(), current_depth + 1, parser.clone()); // 递归调用，增加深度

                            // 通知主线程刷新进度条
                            let mut task_info = self.task_info.lock().unwrap();
                            self.sender.send(task_info.clone()).unwrap(); // 通知主线程任务进度变化
                        }
                    }
                }
            }
            Err(e) => error!("Error fetching URL {}: {}", url, e),
        }
    }



}
