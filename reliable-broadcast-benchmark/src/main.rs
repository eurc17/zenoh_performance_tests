use anyhow::Result;
use clap::Parser;
use output_config::Cli;
use reliable_broadcast as rb;

mod exp;

#[derive(Parser)]
pub struct Opts {
    pub extra_rounds: usize,
    pub sub_mode: rb::SubMode,
    pub reliability: rb::Reliability,
    pub congestion_control: rb::CongestionControl,
    /// The interval between echos in milliseconds
    pub echo_interval: u64,
    pub max_rounds: usize,
    #[clap(flatten)]
    pub cli: Cli,
}

#[async_std::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let opts = Opts::parse();
    exp::run(&opts).await?;
    Ok(())
}
