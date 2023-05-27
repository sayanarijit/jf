use jf::VERSION;
use std::env::Args;
use std::iter::{Peekable, Skip};

#[derive(Debug)]
enum Format {
    Raw,
    Json,
    PrettyJson,
    Yaml,
}

#[derive(Debug)]
enum Cli {
    Help,
    Version,
    Format(Format, Peekable<Skip<Args>>),
}

impl Cli {
    fn parse() -> Result<Self, jf::Error> {
        let mut format = Format::Json;
        let mut args = std::env::args().skip(1).peekable();

        while let Some(arg) = args.peek() {
            match arg.as_str() {
                "-h" | "--help" => return Ok(Self::Help),
                "-v" | "--version" => return Ok(Self::Version),
                "-r" | "--raw" => {
                    format = Format::Raw;
                    args.next();
                }
                "-p" | "--pretty" => {
                    format = Format::PrettyJson;
                    args.next();
                }
                "-y" | "--yaml" => {
                    format = Format::Yaml;
                    args.next();
                }

                "--" => {
                    args.next();
                    break;
                }
                a if a.starts_with('-') => {
                    return Err(format!("invalid argument {a}, try -h or --help")
                        .as_str()
                        .into())
                }
                _ => break,
            }
        }

        Ok(Self::Format(format, args))
    }
}

fn handle(cli: Cli) -> Result<String, jf::Error> {
    match cli {
        Cli::Help => Ok(jf::USAGE.into()),
        Cli::Version => Ok(format!("jf {VERSION}")),
        Cli::Format(Format::Raw, args) => jf::render(args.map(Into::into)),
        Cli::Format(Format::Json, args) => jf::format(args.map(Into::into)),
        Cli::Format(Format::PrettyJson, args) => jf::format_pretty(args.map(Into::into)),
        Cli::Format(Format::Yaml, args) => jf::format_yaml(args.map(Into::into)),
    }
}

fn main() {
    let res = Cli::parse().and_then(handle);
    match res {
        Ok(v) => println!("{v}"),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(e.returncode());
        }
    }
}
