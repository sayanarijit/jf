use serde_json as json;
use serde_yaml as yaml;
use std::{env, fmt::Display};

const USAGE: &str = r#"not enough arguments

USAGE: jf TEMPLATE [VALUE]...

  Where TEMPLATE may contain the following placeholders:

  `%q`: Placeholder for quoted and safely escaped JSON string.
  `%s`: Placeholder for JSON values other than string.

  And [VALUE]... are the values for the placeholders.

  Use `%%` to escape a literal `%` character.

EXAMPLE:

  $ jf '{one: %q, two: %s, "%%": %q}' one 2 100%
  {"one":"one","two":2,"%":"100%"}
"#;

#[derive(Debug)]
enum Error {
    Json(json::Error),
    Yaml(yaml::Error),
    Jf(String),
}

impl Error {
    fn returncode(&self) -> i32 {
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

fn format<I>(args: I) -> Result<String, Error>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    let Some(format) = args.next() else {
        return Err(USAGE.into());
    };

    let mut val = "".to_string();
    let mut last_char = None;

    for (col, ch) in format.chars().enumerate() {
        match (ch, last_char) {
            ('%', Some('%')) => {
                val.push(ch);
                last_char = None;
            }
            ('%', _) => {
                last_char = Some(ch);
            }
            ('q', Some('%')) => {
                let Some(arg) = args.next() else {
                    return Err(format!("placeholder missing value at column {col}").as_str().into())
                };
                val.push_str(&json::to_string(&arg)?);
                last_char = None;
            }
            ('s', Some('%')) => {
                let Some(arg) = args.next() else {
                    return Err(format!("placeholder missing value at column {col}").as_str().into())
                };
                val.push_str(&arg);
                last_char = None;
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

    if last_char == Some('%') {
        return Err("template ended with incomplete placeholder".into());
    };

    if args.count() != 0 {
        return Err("too many values, not enough placeholders".into());
    };

    let val: yaml::Value = yaml::from_str(&val).map_err(Error::from)?;
    json::to_string(&val).map_err(Error::from)
}

fn main() {
    let args = env::args().skip(1).into_iter().map(String::from);

    match format(args) {
        Ok(v) => println!("{}", v),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(e.returncode());
        }
    }
}

#[test]
fn test_format() {
    let args = [
        r#"{"1": %s, one: %q, "true": %s, "truestr": %q, foo: %s, bar: %q, esc: "%%"}"#,
        "1",
        "1",
        "true",
        "true",
        "foo",
        "bar",
    ]
    .into_iter()
    .map(Into::into);

    assert_eq!(
        format(args).unwrap(),
        r#"{"1":1,"one":"1","true":true,"truestr":"true","foo":"foo","bar":"bar","esc":"%"}"#
    );
}

#[test]
fn test_missing_value_error() {
    let args = [
        r#"{"1": %s, one: %q, "true": %s, "truestr": %q, foo: %s, bar: %q, esc: %%}"#,
        "1",
        "1",
        "true",
        "true",
        "foo",
    ]
    .into_iter()
    .map(Into::into);

    assert_eq!(
        format(args).unwrap_err().to_string(),
        "jf: placeholder missing value at column 61"
    );
}

#[test]
fn test_too_many_values_error() {
    let args = [
        r#"{"1": %s, one: %q, "true": %s, "truestr": %q, foo: %s, bar: %q, esc: %%}"#,
        "1",
        "1",
        "true",
        "true",
        "foo",
        "bar",
        "baz",
    ]
    .into_iter()
    .map(Into::into);

    assert_eq!(
        format(args).unwrap_err().to_string(),
        "jf: too many values, not enough placeholders"
    );
}

#[test]
fn test_invalid_placeholder_error() {
    let args = ["foo: %z", "bar"].into_iter().map(Into::into);

    assert_eq!(
        format(args).unwrap_err().to_string(),
        "jf: invalid placeholder '%z' at column 6, use one of '%s' or '%q', or escape it using '%%'"
    );
}

#[test]
fn test_incomplete_placeholder_error() {
    let args = ["foo: %", "bar"].into_iter().map(Into::into);

    assert_eq!(
        format(args).unwrap_err().to_string(),
        "jf: template ended with incomplete placeholder"
    );
}

#[test]
fn test_not_enough_arguments_error() {
    assert_eq!(format([]).unwrap_err().to_string(), format!("jf: {USAGE}"));
}

#[test]
fn test_yaml_error() {
    let args = ["{}{}"].into_iter().map(Into::into);

    assert_eq!(
        format(args).unwrap_err().to_string(),
        "yaml: deserializing from YAML containing more than one document is not supported",
    );
}
