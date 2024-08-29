use crate::config::Config;
use crate::fetcher::Fetcher;
use crate::parser::Parser;
use crate::writer::Writer;
use crate::auth::Authenticator;
use crate::aspect::Aspect;

pub struct Crawler {
    config: Config,
    fetcher: Fetcher,
    parser: Parser,
    writer: Writer,
    authenticator: Option<Authenticator>,
    aspects: Vec<Box<dyn Aspect>>,
}

impl Crawler {
    pub fn new(config: Config) -> Self {
        let authenticator = if let Some(login_config) = &config.login {
            Some(Authenticator::new(login_config))
        } else {
            None
        };

        Crawler {
            config,
            fetcher: Fetcher::new(config.proxy.clone()),
            parser: Parser::new(),
            writer: Writer::new(),
            authenticator,
            aspects: Vec::new(),
        }
    }

    pub fn add_aspect<A: Aspect + 'static>(&mut self, aspect: A) {
        self.aspects.push(Box::new(aspect));
    }

    pub fn run(&mut self) {
        if let Some(auth) = &self.authenticator {
            auth.login();
        }

        for url in &self.config.start_urls {
            let task = CrawlTask::new(url.to_string(), self.config.max_depth);
            Worker::new(self.fetcher.clone(), self.parser.clone(), self.writer.clone()).execute(task);
        }
    }
}
