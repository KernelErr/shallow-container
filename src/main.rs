use clap::{App, load_yaml};
use moe_logger::LogConfig;
use log::{debug, error};

mod models;
mod storage;
mod commands;
use commands::pull;
use commands::run;

#[tokio::main]
async fn main() {
    let log_config = LogConfig::default();
    moe_logger::init(log_config);

    let clap_yaml = load_yaml!("clap.yaml");
    let matches = App::from(clap_yaml).get_matches();

    debug!("{:?}", matches);
    
    let res = match matches.subcommand() {
        Some(("pull",  sub_m)) => pull(sub_m).await,
        Some(("run",   sub_m)) => run(sub_m).await,
        _ => false
    };

    if !res {
        error!("Command exited with error");
    }
}
