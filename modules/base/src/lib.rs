mod config;
mod error;
mod info;
mod prelude;
pub mod shared;
pub mod util;
mod world;

mod result {
    #[allow(unused)]
    pub type Result<T> = std::result::Result<T, crate::error::Error>;
}

pub use config::*;
pub use error::*;
pub use info::*;
pub use result::*;
pub use world::*;
