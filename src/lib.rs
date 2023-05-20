pub use serde_json as json;
pub use serde_yaml as yaml;
use std::{borrow::Cow, collections::HashMap, fmt::Display};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const USAGE: &str = r#"
USAGE

  jf TEMPLATE [VALUE]... [NAME=VALUE]...

  Where TEMPLATE may contain the following placeholders:

  `%q`  quoted and safely escaped JSON string.
  `%s`  JSON values other than string.
  `%v`  the `jf` version number.
  `%%`  a literal `%` character.

  And [VALUE]... [NAME=VALUE]... are the values for the placeholders.

SYNTAX

  `%s`, `%q`                             posiitonal placeholder.
  `%(NAME)s`, `%(NAME)q`                 named placeholder.
  `%(NAME=DEFAULT)s`, `%(NAME=DEFAULT)q` placeholder with default value.
  `%(NAME)?s`, `%(NAME)?q`               placeholder with optional value.
  `%*s`, `%*q`                           expand positional values as array items.
  `%**s`, `%**q`                         expand positional values as key value pairs.
  `%(NAME)*s`, `%(NAME)*q`               expand named values as array items.
  `%(NAME)**s`, `%(NAME)**q`             expand named values as key value pairs.

RULES

  * Pass values for positional placeholders in the same order as in the template.
  * Pass values for named placeholders using `NAME=VALUE` syntax.
  * Pass values for named array items using `NAME=ITEM_N` syntax.
  * Pass values for named key value pairs using `NAME=KEY_N NAME=VALUE_N` syntax.
  * Optional placeholders default to empty string, which is considered as null.
  * Do not declare or pass positional placeholders or values after named ones.
  * Expandable positional placeholder should be the last placeholder in a template.

EXAMPLES

  $ jf %s 1
  - 1

  $ jf %q 1
  - "1"

  $ jf [%*s] 1 2 3
  - [1,2,3]

  $ jf {%**q} one 1 two 2 three 3
  - {"one":"1","two":"2","three":"3"}

  $ jf "{%q: %(value=default)q, %(bar)**q}" foo value=bar bar=biz bar=baz
  - {"foo":"bar","biz":"baz"}

  $ jf "{str_or_bool: %(str)?q %(bool)?s, optional: %(optional)?q}" str=true
  - {"str_or_bool":"true","optional":null}

  $ jf '{1: %s, two: %q, 3: %(3)s, four: %(four=4)q, "%%": %(pct)q}' 1 2 3=3 pct=100%
  - {"1":1,"two":"2","3":3,"four":"4","%":"100%"}
"#;

#[derive(Debug)]
pub enum Error {
    Json(json::Error),
    Yaml(yaml::Error),
    Jf(String),
    Usage,
}

impl Error {
    pub fn returncode(&self) -> i32 {
        match self {
            Self::Usage | Self::Jf(_) => 1,
            Self::Json(_) => 2,
            Self::Yaml(_) => 3,
        }
    }
}

impl From<yaml::Error> for Error {
    fn from(v: yaml::Error) -> Self {
        Self::Yaml(v)
    }
}

impl From<json::Error> for Error {
    fn from(v: json::Error) -> Self {
        Self::Json(v)
    }
}

impl From<&str> for Error {
    fn from(v: &str) -> Self {
        Self::Jf(v.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage => {
                writeln!(f, "jf: not enough arguments")?;
                write!(f, "{USAGE}")
            }

            Self::Json(e) => write!(f, "json: {e}"),
            Self::Yaml(e) => write!(f, "yaml: {e}"),
            Self::Jf(e) => write!(f, "jf: {e}"),
        }
    }
}

fn read_default_value<C>(chars: &mut C) -> String
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

fn read_named_placeholder<C>(
    val: &mut String,
    chars: &mut C,
    named_values: &HashMap<String, Vec<String>>,
) -> Result<(), Error>
where
    C: Iterator<Item = (usize, char)>,
{
    // Reading a named placeholder

    let mut last_char = None;
    let mut name = "".to_string();
    let mut default_value: Option<String> = None;
    let mut is_optional = false;
    let mut is_reading_expandable_items = false;
    let mut is_reading_expandable_pairs = false;

    loop {
        let Some((col, ch)) = chars.next() else {
            return Err("template ended with incomplete placeholder".into());
        };

        match (ch, last_char) {
            ('=', _) => {
                default_value = Some(read_default_value(chars));
                last_char = Some(')');
            }
            (')', _) => {
                last_char = Some(ch);
            }
            ('?', Some(')')) => {
                if default_value.is_some() {
                    return Err(format!("optional placeholder '{name}' at column {col} cannot have a default value").as_str().into());
                }
                is_optional = true;
            }
            ('*', Some(')')) => {
                is_reading_expandable_items = true;
                is_reading_expandable_pairs = false;
                last_char = Some(ch);
            }
            ('*', Some('*')) => {
                is_reading_expandable_pairs = true;
                is_reading_expandable_items = false;
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

                if is_reading_expandable_pairs {
                    read_positional_pairs_placeholder(val, ch, col, &mut args)?;
                } else if is_reading_expandable_items {
                    read_positional_items_placeholder(val, ch, &mut args)?;
                } else {
                    unreachable!();
                }
                break;
            }
            (ch, None) if ch.is_alphanumeric() || ch == '_' => {
                name.push(ch);
                last_char = None;
            }
            (_, Some(')')) => {
                return Err(
                    format!("invalid named placeholder '%({name}){ch}' at column {col}, use '%({name})q' for quoted strings and '%({name})s' for other values")
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

    Ok(())
}

fn read_positional_placeholder<'a, A>(
    val: &mut String,
    ch: char,
    col: usize,
    args: &mut A,
) -> Result<(), Error>
where
    A: Iterator<Item = (usize, Cow<'a, str>)>,
{
    let Some((_, arg)) = args.next() else {
        return Err(format!("placeholder missing value at column {col}").as_str().into())
    };

    if ch == 'q' {
        val.push_str(&json::to_string(&arg)?);
    } else {
        val.push_str(&arg);
    };
    Ok(())
}

fn collect_named_values<'a, A>(
    args: &mut A,
    named_values: &mut HashMap<String, Vec<String>>,
) -> Result<(), Error>
where
    A: Iterator<Item = (usize, Cow<'a, str>)>,
{
    for (valnum, arg) in args.by_ref() {
        let Some((name, value)) = arg.split_once('=') else {
            return Err(format!("invalid syntax for value no. {valnum}, use 'NAME=VALUE' syntax").as_str().into());
        };

        if let Some(values) = named_values.get_mut(name) {
            values.push(value.to_string());
        } else {
            named_values.insert(name.to_string(), vec![value.to_string()]);
        }
    }
    Ok(())
}

fn read_positional_items_placeholder<'a, A>(
    val: &mut String,
    ch: char,
    args: &mut A,
) -> Result<(), Error>
where
    A: Iterator<Item = (usize, Cow<'a, str>)>,
{
    let mut is_empty = true;
    for (_, arg) in args {
        is_empty = false;
        if ch == 'q' {
            val.push_str(&json::to_string(&arg)?);
        } else {
            val.push_str(&arg);
        };
        val.push(',');
    }

    if !is_empty {
        val.pop();
    }
    Ok(())
}

fn read_positional_pairs_placeholder<'a, A>(
    val: &mut String,
    ch: char,
    col: usize,
    args: &mut A,
) -> Result<(), Error>
where
    A: Iterator<Item = (usize, Cow<'a, str>)>,
{
    let mut is_reading_key = true;
    let mut is_empty = true;
    for (_, arg) in args {
        is_empty = false;
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

    if !is_empty {
        val.pop();
    }
    Ok(())
}

fn format_partial<'a, C, A>(
    chars: &mut C,
    args: &mut A,
) -> Result<(String, Option<char>), Error>
where
    C: Iterator<Item = (usize, char)>,
    A: Iterator<Item = (usize, Cow<'a, str>)>,
{
    let mut val = "".to_string();
    let mut last_char = None;
    let mut is_reading_named_values = false;
    let mut named_values = HashMap::<String, Vec<String>>::new();
    let mut is_reading_positional_items = false;
    let mut is_reading_positional_pairs = false;

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
            ('v', Some('%')) => {
                val.push_str(VERSION);
                last_char = None;
            }
            (ch, Some('%')) | (ch, Some('*')) if ch == 's' || ch == 'q' => {
                if is_reading_named_values {
                    return Err(
                        format!("positional placeholder '%{ch}' at column {col} was used after named placeholders, use named placeholder syntax '%(NAME){ch}' instead")
                        .as_str()
                        .into()
                    );
                };

                if is_reading_positional_items {
                    read_positional_items_placeholder(&mut val, ch, args)?;
                    is_reading_positional_items = false;
                } else if is_reading_positional_pairs {
                    read_positional_pairs_placeholder(&mut val, ch, col, args)?;
                    is_reading_positional_pairs = false;
                } else {
                    read_positional_placeholder(&mut val, ch, col, args)?;
                }
                last_char = None;
            }
            ('(', Some('%')) => {
                if !is_reading_named_values {
                    is_reading_named_values = true;
                    collect_named_values(args, &mut named_values)?;
                };
                read_named_placeholder(&mut val, chars, &named_values)?;
                last_char = None;
            }
            ('*', Some('%')) => {
                is_reading_positional_items = true;
                last_char = Some(ch);
            }
            ('*', Some('*')) if is_reading_positional_items => {
                is_reading_positional_items = false;
                is_reading_positional_pairs = true;
                last_char = Some(ch);
            }
            (_, Some('%')) => {
                return Err(format!("invalid placeholder '%{ch}' at column {col}, use one of '%s' or '%q', or escape it using '%%'").as_str().into());
            }
            (_, _) => {
                val.push(ch);
                last_char = None;
                is_reading_positional_items = false;
                is_reading_positional_pairs = false;
            }
        }
    }

    Ok((val, last_char))
}

/// Format a string using the given arguments.
pub fn format<'a, I>(args: I) -> Result<String, Error>
where
    I: IntoIterator<Item = Cow<'a, str>>,
{
    let mut args = args.into_iter().enumerate();
    let Some((_, format)) = args.next() else {
        return Err(Error::Usage);
    };

    let mut chars = format.chars().enumerate();

    let (val, last_char) = format_partial(&mut chars, &mut args)?;

    if last_char == Some('%') {
        return Err("template ended with incomplete placeholder".into());
    };

    if args.count() != 0 {
        return Err(
            "too many positional values, not enough positional placeholders".into(),
        );
    };

    let val: yaml::Value = yaml::from_str(&val).map_err(Error::from)?;
    json::to_string(&val).map_err(Error::from)
}
