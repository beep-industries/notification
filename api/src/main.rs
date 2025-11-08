use std::sync::Arc;

use beep_server::{args::log::LogArgs, get_addr, run_server};
use clap::Parser;
use tracing_subscriber::EnvFilter;

use crate::{args::Args, router::router, state::state};

pub mod args;
pub mod handlers;
pub mod router;
pub mod state;

fn init_logger(args: &LogArgs) {
    let filter = EnvFilter::try_new(&args.filter).unwrap_or_else(|err| {
        eprintln!("invalid log filter: {err}");
        eprintln!("using default log filter: info");
        EnvFilter::new("info")
    });

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr);

    if args.json {
        subscriber.json().init();
    } else {
        subscriber.init();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let args = Arc::new(Args::parse());
    init_logger(&args.log);

    let app_state = state(args.clone()).await?;

    let router = router(app_state)?;

    let addr = get_addr(&args.server.host, args.server.port)
        .await
        .expect("failed to get socket address");

    run_server(addr, router).await;

    Ok(())
}
