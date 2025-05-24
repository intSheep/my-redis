mod server;

use clap::Error;

pub const DEFAULT_PORT:u16 = 6379;

pub type Result<T> =std::result::Result<T,Error>;