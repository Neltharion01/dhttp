//! Utility functions

pub mod httpdate;
pub mod path;
pub(crate) mod escape;
pub(crate) mod future;

mod hex;
pub(crate) use hex::hex;
