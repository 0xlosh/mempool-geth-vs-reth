use eyre::Result;
use tracing::metadata::LevelFilter;
use clap::{
    ArgAction, 
    Args, 
    Parser, 
};
use ethers::prelude::*;
use crate::{runner, hwi::*};

pub async fn run() -> Result<()> {
    let opt = Cli::parse();

    tracing_subscriber::fmt()
        .with_max_level(opt.verbosity.level())
        .without_time()
        .init();

    let geth_url = opt.geth_rpc.as_ref().unwrap();
    if geth_url.starts_with("http") {
        return Err(eyre::eyre!("geth rpc must be ws or ipc"));
    }

    let reth_url = opt.reth_rpc.as_ref().unwrap();
    if reth_url.starts_with("http") {
        return Err(eyre::eyre!("reth rpc must be ws or ipc"));
    }

    let geth = Provider::new(HWI::connect(geth_url).await?);
    let reth = Provider::new(HWI::connect(reth_url).await?);
    
    runner::execute(geth, reth, opt.count).await
}

#[derive(Debug, Parser)]
#[command(author, version = env!("CARGO_PKG_VERSION"), long_about = None)]
struct Cli {
    #[clap(long, global = true, default_value = "ws://127.0.0.1:8546", help = "ws/ipc path")]
    pub geth_rpc: Option<String>,

    #[clap(long, global = true, default_value = "ws://127.0.0.1:9546", help = "ws/ipc path")]
    pub reth_rpc: Option<String>,
    
    #[clap(long, default_value_t = 50, help = "number of transactions to watch")]
    pub count: usize,

    #[clap(flatten)]
    verbosity: Verbosity,
}

/// The verbosity settings for the cli.
#[derive(Debug, Copy, Clone, Args)]
#[command()]
struct Verbosity {
    /// Set the minimum log level.
    ///
    /// -v      Errors
    /// -vv     Warnings
    /// -vvv    Info
    /// -vvvv   Debug
    /// -vvvvv  Traces (warning: very verbose!)
    #[clap(short, long, action = ArgAction::Count, global = true, default_value_t = 1)]
    verbosity: u8,

    /// Silence all log output.
    #[clap(long, alias = "silent", short = 'q', global = true)]
    quiet: bool,
}

impl Verbosity {
    fn level(&self) -> LevelFilter {
        if self.quiet {
            LevelFilter::OFF
        } else {
            match self.verbosity - 1 {
                0 => LevelFilter::ERROR,
                1 => LevelFilter::WARN,
                2 => LevelFilter::INFO,
                3 => LevelFilter::DEBUG,
                _ => LevelFilter::TRACE,
            }
        }
    }
}