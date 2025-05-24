use std::num::ParseIntError;
use std::time::Duration;
use clap::{Parser, Subcommand};
use bytes::Bytes;

#[derive(Parser,Debug)]
#[command(
name="my-redis",
version="1.0",
author="intSheep",
about="Issue Redis commands",
)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
    #[arg(id="hostname",long,default_value = "127.0.0.1")]
    host:String,
    port:u16,
}

#[derive(Subcommand,Debug)]
enum  Command{
    Ping {
    /// Message to ping
    msg: Option<Bytes>,
    },
    Get{
        key:String
    },
    Set{
        key :String,
        value:Bytes,

        #[arg(value_parser =duration_from_ms_str)]
        expires:Option<Duration>
    },
    Publish{
        channel:String,
        message:Bytes
    },
    SubScribe{
        channels:Vec<String>
    },
}

fn duration_from_ms_str(src :&str)->Result<Duration,ParseIntError>{
    let ms = src.parse::<u64>()?;
    Ok(Duration::from_millis(ms))
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> my_redis::Result<()>{
    // tracing_subscriber::fmt::try_init()?;
    // 
    // let cli = Cli::parse();
    // let addr= format!("{}:{}",cli.host,cli.port);
    // 
    // 
    Ok(())
}