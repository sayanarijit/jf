#[cfg(test)]
mod tests;

fn main() {
    let args = std::env::args().skip(1).map(Into::into);

    match jf::format(args) {
        Ok(v) => println!("{}", v),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(e.returncode());
        }
    }
}
