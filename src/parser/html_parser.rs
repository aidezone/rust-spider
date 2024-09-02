use select::document::Document;
use select::predicate::Name;
use scraper::{Html, Selector};
use readability::extractor;
use regex::Regex;
use std::error::Error;
use std::io::Cursor;
use reqwest::Url;
use readability::extractor::Product;

#[derive(Clone)]
pub struct Parser;

impl Parser {
    pub fn new() -> Self {
        Parser
    }

    /// 解析 HTML 内容，提取所有 URL，判断是否为内容页，并提取标题和正文内容
    pub fn parse_content(&self, content: &str, url: &Url) -> Result<(Vec<String>, bool, Option<String>, Option<String>), Box<dyn Error>> {
        // 使用scraper解析HTML
        let document = Html::parse_document(content);

        // 用于匹配<a>标签并提取href属性
        let selector = Selector::parse("a").unwrap();
        let mut urls = Vec::new();
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                urls.push(href.to_string());
            }
        }

        // 使用readability来判断是否为内容页
        let mut reader = Cursor::new(content);
        let parsed_content = extractor::extract(&mut reader, url);
        let is_content_page = parsed_content.is_ok();

        let (title, text_content) = if let Ok(parsed) = parsed_content {
            (parsed.title, parsed.text) // `parsed.title` 和 `parsed.text` 是 `String` 类型
        } else {
            (String::new(), String::new()) // 返回空字符串以匹配 `(String, String)` 类型
        };

        Ok((urls, is_content_page, Some(title), Some(text_content)))
    }
}
