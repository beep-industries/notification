use std::sync::Arc;

use beep_server::{args::log::LogArgs, get_addr, run_server};
use clap::Parser;
use core::domain::ports::broker::BrokerService;
use tokio::{signal::ctrl_c, spawn};
use tracing::{error, info};

use tracing_subscriber::EnvFilter;

use crate::{args::Args, router::router, state::state};

pub mod args;
pub mod handlers;
pub mod router;
pub mod state;

fn init_logger(args: &LogArgs) {
    let filter = EnvFilter::try_new(&args.filter).unwrap_or_else(|err| {
        error!("invalid log filter: {err}");
        error!("using default log filter: info");
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

    let app_state = Arc::new(state(args.clone()).await?);

    let app_state_consumers = app_state.clone();
    let consumers_handle =
        spawn(async move { app_state_consumers.service.start_consumers().await });

    let app_state_for_router = (*app_state).clone();

    let router = router(app_state_for_router)?;

    let addr = get_addr(&args.server.host, args.server.port)
        .await
        .expect("failed to get socket address");

    let server_handle = spawn(async move {
        run_server(addr, router).await;
    });

    // Graceful shutdown
    tokio::select! {
        _ = ctrl_c() => {
            info!("Shutdown signal received");
            Ok(())
        },
        result = consumers_handle => {
            match result {
                Ok(Ok(())) => {
                    error!("Consumers task exited");
                    Err("Consumers exited unexpectedly".into())
                },
                Ok(Err(e)) => {
                    error!("Consumers error: {:?}", e);
                    Err("Consumers failed".into())
                },
                Err(e) => {
                    error!("Consumers task panicked: {:?}", e);
                    Err("Consumers task panicked".into())
                }
            }
        },
        result = server_handle => {
            error!("Server exited: {:?}", result);
            Err("Server terminated unexpectedly".into())
        },
    }
}
