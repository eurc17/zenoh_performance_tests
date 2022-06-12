mod common;
mod exp;
mod msg;
mod utils;

use anyhow::{anyhow, Result};
use async_std::fs;
use clap::Parser;
use output_config::Cli;

#[async_std::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let opts = Cli::parse();
    let report = exp::run(&opts).await.map_err(|err| anyhow!("{}", err))?;

    let output_file = opts.output_dir.join(format!(
        "exp_sub_{}_{}-{}-{}-{}-{}-{}.json",
        opts.peer_id,
        opts.total_put_number,
        opts.total_put_number,
        opts.num_msgs_per_peer,
        opts.payload_size,
        opts.round_timeout,
        opts.init_time
    ));

    fs::create_dir_all(&opts.output_dir).await?;
    let text = serde_json::to_string_pretty(&report)?;
    fs::write(&output_file, &text).await?;

    Ok(())
}
