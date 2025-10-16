fn main() {
    env_logger::init();
    if let Err(e) = libvirtmanager::run_main_app() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
