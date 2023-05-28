use crate as jf;
use crate::VERSION;
use std::env::Args;
use std::iter::Skip;
use std::{fs, io};

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
    Format(Format, Option<String>, Skip<Args>),
}

impl Cli {
    pub fn parse() -> jf::Result<Self> {
        let mut format = Format::Json;
        let mut template: Option<String> = None;
        let mut args = std::env::args().skip(1);
        let mut is_file = false;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => return Ok(Self::Help),
                "-v" | "--version" => return Ok(Self::Version),
                "-r" | "--raw" => {
                    format = Format::Raw;
                }
                "-p" | "--pretty" => {
                    format = Format::PrettyJson;
                }
                "-y" | "--yaml" => {
                    format = Format::Yaml;
                }
                "-f" | "--file" => {
                    is_file = true;
                }
                "-" => {
                    is_file = false;
                    template = Some(io::read_to_string(io::stdin().lock())?);
                    break;
                }
                "--" => {
                    break;
                }
                a if a.starts_with('-') => {
                    return Err(format!("invalid argument {a}, try -h or --help")
                        .as_str()
                        .into())
                }
                _ => {
                    template = Some(arg);
                    break;
                }
            }
        }

        if template.is_none() {
            template = args.next()
        }

        if is_file {
            if let Some(tmpl) = template.as_mut() {
                *tmpl = fs::read_to_string(&tmpl)?;
            }
        }

        Ok(Self::Format(format, template, args))
    }

    pub fn process(self) -> Result<String, jf::Error> {
        match self {
            Self::Help => Ok(jf::USAGE.into()),
            Self::Version => Ok(format!("jf {VERSION}")),
            Self::Format(Format::Raw, template, args) => {
                jf::render(template.iter().map(Into::into).chain(args.map(Into::into)))
            }
            Self::Format(Format::Json, template, args) => {
                jf::format(template.iter().map(Into::into).chain(args.map(Into::into)))
            }
            Self::Format(Format::PrettyJson, template, args) => jf::format_pretty(
                template.iter().map(Into::into).chain(args.map(Into::into)),
            ),
            Self::Format(Format::Yaml, template, args) => jf::format_yaml(
                template.iter().map(Into::into).chain(args.map(Into::into)),
            ),
        }
    }
}

pub fn parse_and_process() -> Result<String, jf::Error> {
    Cli::parse()?.process()
}
