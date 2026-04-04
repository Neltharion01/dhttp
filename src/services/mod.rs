//! Various useful HTTP services

mod defaultservice;
pub use defaultservice::DefaultService;
mod router;
pub use router::Router;
mod files;
pub use files::FilesService;

mod log;
pub use log::DefaultLogger;

mod errorpage;
pub use errorpage::ErrorPageHandler;
