use std::env;
use std::process;

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match brausi::run(args) {
        Ok(()) => {}
        Err(error) => {
            eprintln!("brausi: {error}");
            process::exit(1);
        }
    }
}
