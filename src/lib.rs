/// This `jf` library can be embedded into other Rust programs.
/// To use only the templates and values, use one of the
/// `render` or `format_*` functions.
///
/// To handle also the CLI options, use the `jf::cli` module.
pub mod cli;
pub mod error;
pub use error::{Error, Result};
pub use serde_json as json;
pub use serde_yaml as yaml;

use std::io::BufRead;
use std::{borrow::Cow, collections::HashMap};
use std::{fs, io};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const USAGE: &str = include_str!("usage.txt");

fn read_to_string<S>(path: &str, stdin: &mut S) -> Result<String>
where
    S: Iterator<Item = (usize, io::Result<Vec<u8>>)>,
{
    if path == "-" {
        match stdin.next() {
            Some((_, Ok(bytes))) => Ok(String::from_utf8_lossy(&bytes).to_string()),
            Some((_, Err(e))) => Err(e.into()),
            None => Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "unexpected end of input",
            )
            .into()),
        }
    } else {
        fs::read_to_string(path).map_err(Into::into)
    }
}

fn read_brace_value<C>(chars: &mut C) -> String
where
    C: Iterator<Item = (usize, char)>,
{
    // Reading a default value for a named placeholder

    let mut last_char = None;
    let mut val = String::new();

    for (_, ch) in chars {
        match (ch, last_char) {
            (_, Some('\\')) => {
                val.push(ch);
                last_char = None;
            }
            ('\\', _) => {
                last_char = Some(ch);
            }
            (')', _) => {
                break;
            }
            (_, _) => {
                val.push(ch);
                last_char = None;
            }
        }
    }

    val
}

#[derive(Debug, PartialEq, Eq)]
enum Expansion {
    None,
    Items,
    Pairs,
}

impl Expansion {
    fn stars(&self) -> &'static str {
        match self {
            Expansion::None => "",
            Expansion::Items => "*",
            Expansion::Pairs => "**",
        }
    }
}

fn read_named_placeholder<C, S>(
    val: &mut String,
    chars: &mut C,
    named_values: &HashMap<String, Vec<String>>,
    stdin: &mut S,
) -> Result<bool>
where
    C: Iterator<Item = (usize, char)>,
    S: Iterator<Item = (usize, io::Result<Vec<u8>>)>,
{
    // Reading a named placeholder

    let mut last_char = None;
    let mut name = "".to_string();
    let mut default_value: Option<String> = None;
    let mut is_optional = false;
    let mut is_nullable = false;
    let mut expansion = Expansion::None;
    let mut empty_expansion = false;

    loop {
        let Some((col, ch)) = chars.next() else {
            return Err("template ended with incomplete placeholder".into());
        };

        match (ch, last_char) {
            ('=', _) if default_value.is_none() => {
                default_value = Some(read_brace_value(chars));
                last_char = Some(')');
            }
            ('@', _) if default_value.is_none() => {
                let pth = read_brace_value(chars);
                default_value = Some(read_to_string(&pth, stdin)?);
                last_char = Some(')');
            }
            (')', _) => {
                last_char = Some(ch);
            }
            ('?', Some(')')) => {
                if default_value.is_some() {
                    return Err(format!("optional placeholder '{name}' at column {col} cannot have a default value").as_str().into());
                }
                if is_nullable {
                    return Err(format!("optional placeholder '{name}' at column {col} cannot also be nullable").as_str().into());
                }
                is_optional = true;
            }
            ('?', None) => {
                is_nullable = true;
                last_char = chars.next().map(|(_, ch)| ch);
                if last_char != Some(')') {
                    return Err(format!("nullable placeholder '{name}' at column {col} must end with '?)'", col = col).as_str().into());
                }
            }
            ('*', Some(')')) => {
                expansion = Expansion::Items;
                last_char = Some(ch);
            }
            ('*', Some('*')) => {
                expansion = Expansion::Pairs;
                last_char = Some(ch);
            }
            (ch, Some(')')) if ch == 'q' || ch == 's' => {
                if name.is_empty() {
                    return Err(format!("placeholder missing name at column {col}")
                        .as_str()
                        .into());
                }
                let maybe_value = named_values
                    .get(&name)
                    .and_then(|v| v.first())
                    .or(default_value.as_ref());

                if let Some(value) = maybe_value {
                    if ch == 'q' {
                        val.push_str(&json::to_string(value)?);
                    } else {
                        val.push_str(value);
                    }
                } else if is_nullable {
                    val.push_str("null");
                } else if !is_optional {
                    return Err(format!(
                        "no value for placeholder '%({name}){ch}' at column {col}"
                    )
                    .as_str()
                    .into());
                };
                break;
            }

            (ch, Some('*')) if ch == 'q' || ch == 's' => {
                if name.is_empty() {
                    return Err(format!("placeholder missing name at column {col}")
                        .as_str()
                        .into());
                }

                if default_value.is_some() {
                    return Err(format!("expandable placeholder '{name}' at column {col} cannot have a default value").as_str().into());
                }

                let mut args = named_values
                    .get(&name)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .map(Into::into)
                    .enumerate();

                match expansion {
                    Expansion::Items => {
                        empty_expansion = !read_positional_items_placeholder(
                            val, ch, col, false, &mut args, stdin,
                        )?;
                    }
                    Expansion::Pairs => {
                        empty_expansion = !read_positional_pairs_placeholder(
                            val, ch, col, false, &mut args, stdin,
                        )?;
                    }
                    Expansion::None => {
                        unreachable!();
                    }
                }
                break;
            }
            (ch, None) if ch.is_alphanumeric() || ch == '_' => {
                name.push(ch);
                last_char = None;
            }
            (_, Some(')')) | (_, Some('*')) => {
                let stars = expansion.stars();
                return Err(
                    format!("invalid named placeholder '%({name}){stars}{ch}' at column {col}, use '%({name}){stars}q' for quoted strings and '%({name}){stars}s' for other values")
                    .as_str()
                    .into()
                );
            }
            (_, _) => {
                return Err(
                    format!("invalid character {ch:?} in placeholder name at column {col}, use numbers, letters and underscores only")
                    .as_str()
                    .into()
                );
            }
        }
    }

    Ok(empty_expansion)
}

fn collect_named_values<'a, A, S>(
    args: &mut A,
    stdin: &mut S,
    named_values: &mut HashMap<String, Vec<String>>,
) -> Result<()>
where
    A: Iterator<Item = (usize, Cow<'a, str>)>,
    S: Iterator<Item = (usize, io::Result<Vec<u8>>)>,
{
    for (valnum, arg) in args.by_ref() {
        let (name, value) = if let Some((name, value)) = arg.split_once('=') {
            (name, value.to_string())
        } else if let Some((name, path)) = arg.split_once('@') {
            let value = read_to_string(path, stdin)?;
            (name, value)
        } else {
            return Err(format!(
                "invalid syntax for value no. {valnum}, use 'NAME=VALUE' or 'NAME@FILE' syntax"
            )
            .as_str()
            .into());
        };

        if let Some(values) = named_values.get_mut(name) {
            values.push(value.to_string());
        } else {
            named_values.insert(name.to_string(), vec![value.to_string()]);
        }
    }
    Ok(())
}

fn read<'a, A, S>(
    is_stdin: bool,
    col: usize,
    args: &mut A,
    stdin: &mut S,
) -> Result<(usize, String)>
where
    A: Iterator<Item = (usize, Cow<'a, str>)>,
    S: Iterator<Item = (usize, io::Result<Vec<u8>>)>,
{
    let maybe_arg = if is_stdin {
        if let Some((i, arg)) = stdin.next() {
            let arg = arg?;
            let arg = String::from_utf8_lossy(&arg).to_string();
            Some((i, arg))
        } else {
            None
        }
    } else {
        args.next().map(|(i, a)| (i, a.to_string()))
    };

    if let Some((i, arg)) = maybe_arg {
        Ok((i, arg))
    } else {
        Err(format!("placeholder missing value at column {col}")
            .as_str()
            .into())
    }
}

fn read_positional_placeholder<'a, A, S>(
    val: &mut String,
    ch: char,
    col: usize,
    is_stdin: bool,
    args: &mut A,
    stdin: &mut S,
) -> Result<()>
where
    A: Iterator<Item = (usize, Cow<'a, str>)>,
    S: Iterator<Item = (usize, io::Result<Vec<u8>>)>,
{
    let (_, arg) = read(is_stdin, col, args, stdin)?;

    if ch == 'q' {
        val.push_str(&json::to_string(&arg)?);
    } else {
        val.push_str(&arg);
    };
    Ok(())
}

fn read_positional_items_placeholder<'a, A, S>(
    val: &mut String,
    ch: char,
    col: usize,
    is_stdin: bool,
    args: &mut A,
    stdin: &mut S,
) -> Result<bool>
where
    A: Iterator<Item = (usize, Cow<'a, str>)>,
    S: Iterator<Item = (usize, io::Result<Vec<u8>>)>,
{
    let mut was_expanded = false;

    while let Ok((_, arg)) = read(is_stdin, col, args, stdin) {
        was_expanded = true;
        if ch == 'q' {
            val.push_str(&json::to_string(&arg)?);
        } else {
            val.push_str(&arg);
        };
        val.push(',');
    }

    if was_expanded {
        val.pop();
    }
    Ok(was_expanded)
}

fn read_positional_pairs_placeholder<'a, A, S>(
    val: &mut String,
    ch: char,
    col: usize,
    is_stdin: bool,
    args: &mut A,
    stdin: &mut S,
) -> Result<bool>
where
    A: Iterator<Item = (usize, Cow<'a, str>)>,
    S: Iterator<Item = (usize, io::Result<Vec<u8>>)>,
{
    let mut is_reading_key = true;
    let mut was_expanded = false;
    while let Ok((_, arg)) = read(is_stdin, col, args, stdin) {
        was_expanded = true;
        let arg = if is_reading_key || ch == 'q' {
            json::to_string(&arg)?
        } else {
            arg.to_string()
        };

        val.push_str(&arg);

        if is_reading_key {
            val.push(':');
            is_reading_key = false;
        } else {
            val.push(',');
            is_reading_key = true;
        }
    }

    if !is_reading_key {
        return Err(format!("placeholder missing value at column {col}")
            .as_str()
            .into());
    }

    if was_expanded {
        val.pop();
    }
    Ok(was_expanded)
}

fn format_partial<'a, C, A, S>(
    chars: &mut C,
    args: &mut A,
    stdin: &mut S,
) -> Result<(String, Option<char>)>
where
    C: Iterator<Item = (usize, char)>,
    A: Iterator<Item = (usize, Cow<'a, str>)>,
    S: Iterator<Item = (usize, io::Result<Vec<u8>>)>,
{
    let mut val = "".to_string();
    let mut last_char = None;
    let mut is_reading_named_values = false;
    let mut named_values = HashMap::<String, Vec<String>>::new();
    let mut expansion = Expansion::None;
    let mut is_stdin = false;
    let mut empty_expansion = false;

    while let Some((col, ch)) = chars.next() {
        // Reading a named placeholder
        // Not reading a named placeholder
        match (ch, last_char) {
            ('%', Some('%')) => {
                val.push(ch);
                last_char = None;
            }
            ('%', _) => {
                last_char = Some(ch);
            }
            ('(', Some('%')) => {
                if !is_reading_named_values {
                    is_reading_named_values = true;
                    collect_named_values(args, stdin, &mut named_values)?;
                };
                empty_expansion =
                    read_named_placeholder(&mut val, chars, &named_values, stdin)?;
                last_char = None;
            }
            ('*', Some('%')) if expansion == Expansion::None => {
                expansion = Expansion::Items;
                last_char = Some('%');
            }
            ('*', Some('%')) if expansion == Expansion::Items => {
                expansion = Expansion::Pairs;
                last_char = Some('%');
            }
            ('-', Some('%')) => {
                is_stdin = true;
                last_char = Some('%');
            }
            (',', None) if empty_expansion => {
                empty_expansion = false;
                last_char = None;
            }
            (ch, Some('%')) if ch == 's' || ch == 'q' => {
                if is_reading_named_values {
                    return Err(
                        format!("positional placeholder '%{ch}' at column {col} was used after named placeholders, use named placeholder syntax '%(NAME){ch}' instead")
                        .as_str()
                        .into()
                    );
                };

                match expansion {
                    Expansion::Items => {
                        empty_expansion = !read_positional_items_placeholder(
                            &mut val, ch, col, is_stdin, args, stdin,
                        )?;
                        expansion = Expansion::None;
                    }
                    Expansion::Pairs => {
                        empty_expansion = !read_positional_pairs_placeholder(
                            &mut val, ch, col, is_stdin, args, stdin,
                        )?;
                        expansion = Expansion::None;
                    }
                    Expansion::None => {
                        read_positional_placeholder(
                            &mut val, ch, col, is_stdin, args, stdin,
                        )?;
                        empty_expansion = false;
                    }
                }
                is_stdin = false;
                last_char = None;
            }
            (_, Some('%')) => {
                let stars = expansion.stars();
                return Err(format!("invalid placeholder '%{stars}{ch}' at column {col}, use one of '%{stars}s' or '%{stars}q', or escape it using '%%'").as_str().into());
            }
            (_, _) => {
                val.push(ch);
                last_char = None;
                expansion = Expansion::None;
                is_stdin = false;
                empty_expansion = false;
            }
        }
    }

    Ok((val, last_char))
}

/// Render the template into raw string using the given arguments.
pub fn render<'a, I>(args: I) -> Result<String>
where
    I: IntoIterator<Item = Cow<'a, str>>,
{
    let mut args = args.into_iter().enumerate();
    let Some((_, format)) = args.next() else {
        return Err("not enough arguments, expected at least one".into());
    };

    let mut chars = format.chars().enumerate();
    let mut stdin = io::stdin().lock().split(b'\0').enumerate();

    let (val, last_char) = format_partial(&mut chars, &mut args, &mut stdin)?;

    if last_char == Some('%') {
        return Err("template ended with incomplete placeholder".into());
    };

    if args.count() != 0 {
        return Err(
            "too many positional values, not enough positional placeholders".into(),
        );
    };

    Ok(val)
}

/// Render and format the template into JSON.
pub fn format<'a, I>(args: I) -> Result<String>
where
    I: IntoIterator<Item = Cow<'a, str>>,
{
    let val = render(args)?;
    let yaml: yaml::Value = yaml::from_str(&val).map_err(Error::from)?;
    json::to_string(&yaml).map_err(Error::from)
}

/// Render and format the template into pretty JSON.
pub fn format_pretty<'a, I>(args: I) -> Result<String>
where
    I: IntoIterator<Item = Cow<'a, str>>,
{
    let val = render(args)?;
    let yaml: yaml::Value = yaml::from_str(&val).map_err(Error::from)?;
    json::to_string_pretty(&yaml).map_err(Error::from)
}

/// Render and format the template into value JSON using the given arguments.
pub fn format_yaml<'a, I>(args: I) -> Result<String>
where
    I: IntoIterator<Item = Cow<'a, str>>,
{
    let val = render(args)?;
    let yaml: yaml::Value = yaml::from_str(&val).map_err(Error::from)?;
    yaml::to_string(&yaml).map_err(Error::from)
}

#[cfg(test)]
mod tests;
