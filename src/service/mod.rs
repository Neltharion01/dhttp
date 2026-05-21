//! Various useful HTTP services

mod defaultservice;
pub use defaultservice::DefaultService;
mod router;
pub use router::Router;
mod files;
pub use files::Files;
mod redirect;
pub use redirect::Redirect;

mod log;
pub use log::DefaultLogger;

mod errorpage;
pub use errorpage::ErrorPageHandler;
