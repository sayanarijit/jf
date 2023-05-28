fn main() {
    match jf::cli::parse_and_process() {
        Ok(v) => println!("{v}"),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(e.returncode());
        }
    }
}
