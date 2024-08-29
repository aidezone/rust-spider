use select::document::Document;
use select::predicate::Name;

pub struct Parser;

impl Parser {
    pub fn new() -> Self {
        Parser
    }

    pub fn parse(&self, content: &str) -> Vec<String> {
        let document = Document::from(content);
        document.find(Name("a"))
            .filter_map(|n| n.attr("href"))
            .map(|s| s.to_string())
            .collect()
    }
}
