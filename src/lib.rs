//! DrakoHTTP: the best web framework

pub(crate) mod h1;

pub mod reqres;
pub mod core;
pub mod services;
pub mod prelude;
pub mod server;
pub mod util;

pub use server::{tokio_rt, serve_tcp};
