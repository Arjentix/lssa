use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use clap::Parser;
use common::rpc_primitives::RpcConfig;
use log::info;
use node_core::NodeCore;
use sequencer_core::SequencerCore;
use tokio::{sync::Mutex, task::JoinHandle};

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    /// Path to configs
    home_dir: PathBuf,
}

pub async fn main_tests_runner() -> Result<()> {
    env_logger::init();

    let args = Args::parse();
    let Args { home_dir } = args;

    let sequencer_config = sequencer_runner::config::from_file(home_dir.join("sequencer_config.json"))?;
    let node_config = node_runner::config::from_file(home_dir.join("node_config.json"))?;

    let block_timeout = sequencer_config.block_create_timeout_millis;
    let sequencer_port = sequencer_config.port;

    let sequencer_core = SequencerCore::start_from_config(sequencer_config);

    info!("Sequencer core set up");

    let seq_core_wrapped = Arc::new(Mutex::new(sequencer_core));

    let http_server = sequencer_rpc::new_http_server(RpcConfig::with_port(sequencer_port), seq_core_wrapped.clone())?;
    info!("HTTP server started");
    let _http_server_handle = http_server.handle();
    tokio::spawn(http_server);

    info!("Starting main sequencer loop");

    let _sequencer_loop_handle: JoinHandle<Result<()>> = tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(block_timeout)).await;

            info!("Collecting transactions from mempool, block creation");

            let id = {
                let mut state = seq_core_wrapped.lock().await;

                state.produce_new_block_with_mempool_transactions()?
            };

            info!("Block with id {id} created");

            info!("Waiting for new transactions");
        }
    });

    let node_port = node_config.port;

    let node_core = NodeCore::start_from_config_update_chain(node_config.clone()).await?;
    let wrapped_node_core = Arc::new(Mutex::new(node_core));

    let http_server = node_rpc::new_http_server(
        RpcConfig::with_port(node_port),
        node_config.clone(),
        wrapped_node_core.clone(),
    )?;
    info!("HTTP server started");
    let _http_server_handle = http_server.handle();
    tokio::spawn(http_server);

    #[allow(clippy::empty_loop)]
    loop {}
}
