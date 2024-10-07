use anyhow::Result;
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

mod app;
mod cleanup;
mod cli;
mod logger;
mod service;
mod tera;
mod utils;
mod workspace;

static CLEANUP_NEEDED: AtomicBool = AtomicBool::new(false);
static CLEANUP_MANAGER: Lazy<Mutex<cleanup::CleanupManager>> =
    Lazy::new(|| Mutex::new(cleanup::CleanupManager::new()));

fn perform_cleanup() {
    let manager = CLEANUP_MANAGER.lock().unwrap();
    manager.cleanup();
}

pub fn set_cleanup_needed() {
    CLEANUP_NEEDED.store(true, Ordering::SeqCst);
    perform_cleanup();
}

pub fn run() -> Result<()> {
    logger::initialize_logger()?;

    ctrlc::set_handler(|| {
        CLEANUP_NEEDED.store(true, Ordering::SeqCst);
        perform_cleanup();
        std::process::exit(1);
    })?;

    let args: Vec<String> = std::env::args().collect();

    cli::parse_cli(args)?;

    if CLEANUP_NEEDED.load(Ordering::SeqCst) {
        perform_cleanup();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        use cli::Cli;

        Cli::command().debug_assert();
    }
}
