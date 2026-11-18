mod daemon;
mod gallery;
mod shared;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let result = match args.get(1).map(String::as_str) {
        None | Some("gui") => gallery::run(),
        Some("daemon") => daemon::run(),
        Some("shuffle") => shared::shuffle(),
        Some("--help") | Some("-h") => {
            println!("Usage: omarchy-wallpapers [gui|daemon|shuffle]");
            Ok(())
        }
        Some(arg) => Err(format!("Unknown command: {arg}")),
    };
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
