use crate::fetcher::Fetcher;
use crate::parser::Parser;
use crate::writer::Writer;
use crate::crawler::task::CrawlTask;

pub struct Worker {
    fetcher: Fetcher,
    parser: Parser,
    writer: Writer,
}

impl Worker {
    pub fn new(fetcher: Fetcher, parser: Parser, writer: Writer) -> Self {
        Worker { fetcher, parser, writer }
    }

    pub fn execute(&self, task: CrawlTask) {
        if task.depth == 0 {
            return;
        }

        let content = self.fetcher.fetch(&task.url);
        let links = self.parser.parse(&content);

        self.writer.write(&task.url, &content);

        for link in links {
            let new_task = CrawlTask::new(link, task.depth - 1);
            self.execute(new_task);
        }
    }
}
