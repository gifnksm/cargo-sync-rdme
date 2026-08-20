pub(in crate::sync) use self::{replace::*, resolve::*, scan::*};

mod parse;
mod replace;
mod resolve;
mod scan;

const MAGIC: &str = "cargo-sync-rdme";
