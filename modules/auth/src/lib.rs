pub mod crypto;
pub mod email;
mod error;
pub mod model;
mod prelude;
pub mod provider;
pub mod token;
pub mod user;

mod result {
    #[allow(unused)]
    pub type Result<T> = std::result::Result<T, crate::error::Error>;
}

pub use error::*;
pub use result::*;
