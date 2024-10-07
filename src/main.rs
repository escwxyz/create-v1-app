use anyhow::Result;
use create_v1_app::{run, set_cleanup_needed};

fn main() -> Result<()> {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        set_cleanup_needed();
        std::process::exit(1);
    }
    Ok(())
}
