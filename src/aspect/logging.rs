use crate::aspect::Aspect;

pub struct LoggingAspect;

impl Aspect for LoggingAspect {
    fn before(&self) {
        println!("Start crawling...");
    }

    fn after(&self) {
        println!("Finished crawling.");
    }
}
