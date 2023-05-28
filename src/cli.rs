use crate as jf;
use crate::VERSION;
use std::env::Args;
use std::io;
use std::iter::{Peekable, Skip};

#[derive(Debug)]
pub enum Format {
    Raw,
    Json,
    PrettyJson,
    Yaml,
}

#[derive(Debug)]
pub enum Cli {
    Help,
    Version,
    Format(Format, Peekable<Skip<Args>>),
}

impl Cli {
    pub fn parse() -> Result<Self, jf::Error> {
        let mut format = Format::Json;
        let mut args = std::env::args().skip(1).peekable();

        while let Some(arg) = args.peek_mut() {
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
                "-" => {
                    *arg = io::read_to_string(io::stdin().lock())?;
                    break;
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

    pub fn process(self) -> Result<String, jf::Error> {
        match self {
            Self::Help => Ok(jf::USAGE.into()),
            Self::Version => Ok(format!("jf {VERSION}")),
            Self::Format(Format::Raw, args) => jf::render(args.map(Into::into)),
            Self::Format(Format::Json, args) => jf::format(args.map(Into::into)),
            Self::Format(Format::PrettyJson, args) => {
                jf::format_pretty(args.map(Into::into))
            }
            Self::Format(Format::Yaml, args) => jf::format_yaml(args.map(Into::into)),
        }
    }
}

pub fn parse_and_process() -> Result<String, jf::Error> {
    Cli::parse()?.process()
}
