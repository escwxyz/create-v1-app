use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod cli;
mod logger;
mod tera;
mod utils;

static CLEANUP_NEEDED: AtomicBool = AtomicBool::new(false);

fn cleanup() {
    if CLEANUP_NEEDED.load(Ordering::SeqCst) {
        logger::log_debug("Cleaning up...");
        logger::log_debug("Cleanup completed.");
    }
}

pub fn run(args: Vec<String>) -> Result<()> {
    logger::initialize_logger()?;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        logger::log_debug("Received Ctrl+C, initiating graceful shutdown...");
        r.store(false, Ordering::SeqCst);
        CLEANUP_NEEDED.store(true, Ordering::SeqCst);
        cleanup();
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    CLEANUP_NEEDED.store(true, Ordering::SeqCst);
    let result = cli::parse_cli(args);

    // Perform cleanup if needed
    cleanup();

    result
}
