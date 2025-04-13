mod converter;
mod error;
mod executor;
mod job_creator;
mod message;
mod prelude;
mod repo;

mod result {
    #[allow(unused)]
    pub type Result<T> = std::result::Result<T, crate::error::Error>;
}

pub use converter::*;
pub use error::*;
pub use executor::*;
pub use job_creator::*;
pub use message::*;
pub use repo::*;
pub use result::*;
