mod common;
mod exp;
mod msg;
mod utils;

use anyhow::{anyhow, Result};
use clap::Parser;
use output_config::Cli;

#[async_std::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let opts = Cli::parse();
    exp::run(&opts).await.map_err(|err| anyhow!("{}", err))?;
    Ok(())
}
