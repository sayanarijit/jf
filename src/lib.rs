pub use serde_json as json;
pub use serde_yaml as yaml;
use std::{borrow::Cow, collections::HashMap, fmt::Display};

pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) const USAGE: &str = r#"not enough arguments

USAGE: jf TEMPLATE [VALUE]... [NAME=VALUE]...

  Where TEMPLATE may contain the following placeholders:

  %q, %(NAME)q, %(NAME=DEFAULT)q for quoted and safely escaped JSON string.
  %s, %(NAME)s, %(NAME=DEFAULT)s for JSON values other than string.

  And [VALUE]... [NAME=VALUE]... are the values for the placeholders.

  Use `%s` or `%q` syntax to declare positional placeholders.
  Use `%(NAME)s` or `%(NAME)q` syntax to declare named placeholders.
  Use `%(NAME=DEFAULT)s` or `%(NAME=DEFAULT)q` syntax to declare default values for named placeholders.
  Use `%%` to escape a literal `%` character.
  Pass values for positional placeholders in the same order as in the template.
  Pass values for named placeholders using `NAME=VALUE` syntax.
  Do not declare or pass positional placeholders or values after named ones.
  To get the `jf` version number, run `jf %v`.

EXAMPLE:

  $ jf '{1: %s, two: %q, 3: %(3)s, four: %(four=4)q, "%%": %(pct)q}' 1 2 3=3 pct=100%
  {"1":1,"two":"2","3":3,"four":"4","%":"100%"}
"#;

#[derive(Debug)]
pub enum Error {
    Json(json::Error),
    Yaml(yaml::Error),
    Jf(String),
}

impl Error {
    pub fn returncode(&self) -> i32 {
        match self {
            Self::Jf(_) => 1,
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
            Self::Json(e) => write!(f, "json: {e}"),
            Self::Yaml(e) => write!(f, "yaml: {e}"),
            Self::Jf(e) => write!(f, "jf: {e}"),
        }
    }
}

pub fn format<'a, I>(args: I) -> Result<String, Error>
where
    I: IntoIterator<Item = Cow<'a, str>>,
{
    let mut args = args.into_iter().enumerate();
    let Some((_, format)) = args.next() else {
        return Err(USAGE.into());
    };

    let mut val = "".to_string();
    let mut last_char = None;
    let mut is_reading_named_placeholders = false;
    let mut placeholder_name: Option<String> = None;
    let mut named_placeholders = HashMap::<String, String>::new();
    let mut default_value: Option<String> = None;

    for (col, ch) in format.chars().enumerate() {
        if let Some(name) = placeholder_name.as_mut() {
            // Reading a named placeholder

            if let Some(val) = default_value.as_mut() {
                // Reading a default value for a named placeholder

                match (ch, last_char) {
                    (ch, Some('\\')) => {
                        val.push(ch);
                        last_char = None;
                    }
                    ('\\', _) => {
                        last_char = Some(ch);
                    }
                    (')', _) => {
                        if !named_placeholders.contains_key(name) {
                            named_placeholders.insert(name.to_string(), val.to_string());
                        }
                        default_value = None;
                        last_char = Some(ch);
                    }
                    (ch, _) => {
                        val.push(ch);
                        last_char = Some(ch);
                    }
                }
            } else {
                match (ch, last_char) {
                    ('=', _) => {
                        default_value = Some("".into());
                        last_char = None;
                    }
                    ('q', Some(')')) => {
                        let value = named_placeholders.get(name).ok_or(
                            format!(
                                "no value for placeholder '%({name})q' at column {col}"
                            )
                            .as_str(),
                        )?;
                        val.push_str(&json::to_string(value)?);
                        placeholder_name = None;
                        last_char = None;
                    }
                    ('s', Some(')')) => {
                        let value = named_placeholders.get(name).ok_or(
                            format!(
                                "no value for placeholder '%({name})s' at column {col}"
                            )
                            .as_str(),
                        )?;
                        val.push_str(value);
                        placeholder_name = None;
                        last_char = None;
                    }
                    (ch, Some(')')) => {
                        return Err(format!(
                        "invalid named placeholder '%({name}){ch}' at column {col}, use '%({name})q' for quoted strings and '%({name})s' for other values"
                    )
                    .as_str()
                    .into());
                    }
                    (')', _) => {
                        last_char = Some(ch);
                    }
                    (ch, _) if ch.is_alphanumeric() || ch == '_' => {
                        name.push(ch);
                        last_char = Some(ch);
                    }
                    (ch, _) => {
                        return Err(format!(
                        "invalid character {ch:?} in placeholder name at column {col}, use numbers, letters and underscores only"
                    )
                    .as_str()
                    .into());
                    }
                }
            }
        } else {
            // Not reading a named placeholder
            match (ch, last_char) {
                ('%', Some('%')) => {
                    val.push(ch);
                    last_char = None;
                }
                ('%', _) => {
                    last_char = Some(ch);
                }
                ('q', Some('%')) => {
                    if is_reading_named_placeholders {
                        return Err(
                        format!("positional placeholder '%q' at column {col} was used after named placeholders, use named placeholder syntax '%(NAME)q' instead")
                            .as_str()
                            .into()
                        );
                    };

                    let Some((_, arg)) = args.next() else {
                        return Err(format!("placeholder missing value at column {col}")
                            .as_str()
                            .into())
                    };

                    val.push_str(&json::to_string(&arg)?);
                    last_char = None;
                }
                ('s', Some('%')) => {
                    if is_reading_named_placeholders {
                        return Err(
                        format!("positional placeholder '%s' at column {col} was used after named placeholders, use named placeholder syntax '%(NAME)s' instead")
                            .as_str()
                            .into()
                        );
                    };

                    let Some((_, arg)) = args.next() else {
                        return Err(format!("placeholder missing value at column {col}")
                            .as_str()
                            .into())
                    };

                    val.push_str(&arg);
                    last_char = None;
                }
                ('v', Some('%')) => {
                    val.push_str(&VERSION);
                    last_char = None;
                }
                ('(', Some('%')) => {
                    if !is_reading_named_placeholders {
                        is_reading_named_placeholders = true;
                        while let Some((valnum, arg)) = args.next() {
                            let Some((name, value)) = arg.split_once('=') else {
                                return Err(format!("invalid syntax for value no. {valnum}, use 'NAME=VALUE' syntax").as_str().into());
                            };
                            named_placeholders
                                .insert(name.to_string(), value.to_string());
                        }
                    };
                    placeholder_name = Some("".to_string());
                }
                (ch, Some('%')) => {
                    return Err(format!("invalid placeholder '%{ch}' at column {col}, use one of '%s' or '%q', or escape it using '%%'").as_str().into());
                }
                (ch, _) => {
                    val.push(ch);
                    last_char = Some(ch);
                }
            }
        }
    }

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
