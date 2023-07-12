use eyre::Result;
use tracing::metadata::LevelFilter;
use clap::{
    ArgAction, 
    Args, 
    Parser, 
};
use ethers::types::Chain;
use crate::runner;

pub async fn run() -> Result<()> {
    let mut opt = Cli::parse();
    opt.ctx.init()?;

    tracing_subscriber::fmt()
        .with_max_level(opt.verbosity.level())
        .without_time()
        .init();

    runner::execute(opt.ctx).await
}

#[derive(Debug, Parser)]
#[command(author, version = env!("CARGO_PKG_VERSION"), long_about = None)]
struct Cli {
    #[clap(flatten)]
    ctx: Ctx,
    
    #[clap(flatten)]
    verbosity: Verbosity,
}

#[derive(Debug, Clone, Args)]
#[command()]
pub struct Ctx {
    #[clap(long, short = 'f', default_value_t = Chain::Mainnet, global = true, conflicts_with = "rpc")]
    network: Chain,

    #[clap(long, global = true, conflicts_with = "network")]
    pub rpc: Option<String>,

    #[clap(long= "gasprice", alias = "gp", global = true)]
    pub gasprice: Option<String>,

    #[clap(long= "gas", global = true)]
    pub gas: Option<u64>,
}

impl Ctx {
    fn init(&mut self) -> Result<()> {
        if self.rpc.is_none() {
            let rpc = match self.network {
                Chain::Mainnet => "/data/geth/geth.ipc",
                Chain::Arbitrum => "/data/arbitrum/arb1/arb.ipc",

                _ => return Err(eyre::eyre!("no default rpc for {:?}", self.network)),
            };

            self.rpc = Some(rpc.to_string());
        }

        Ok(())
    }
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