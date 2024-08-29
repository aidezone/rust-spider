mod logging;

pub use self::logging::LoggingAspect;

pub trait Aspect {
    fn before(&self);
    fn after(&self);
}
