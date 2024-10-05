fn main() {
    let args = std::env::args().collect();
    let result: Result<_, _> = create_v1_app::run(args);
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
