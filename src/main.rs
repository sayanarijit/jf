use serde_json as json;
use serde_yaml as yaml;
use std::{collections::HashMap, env, fmt::Display};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const USAGE: &str = r#"not enough arguments

USAGE: jf TEMPLATE [VALUE]... [NAME=VALUE]...

  Where TEMPLATE may contain the following placeholders:

  `%q` or `%(NAME)q`: For quoted and safely escaped JSON string.
  `%s` or `%(NAME)s`: For JSON values other than string.

  And [VALUE]... [[NAME=]VALUE]... are the values for the placeholders.

  Use `%s` or `%q` syntax to declare positional placeholders.
  Use `%(NAME)s` or `%(NAME)q` syntax to declare named placeholders.
  Use `%%` to escape a literal `%` character.
  Pass values for positional placeholders in the same order as in the template.
  Pass values for named placeholders using `NAME=VALUE` syntax.
  Do not declare or pass positional placeholders or values after named ones.
  To get the `jf` version number, use `jf %v`.

EXAMPLE:

  $ jf '{one: %q, two: %s, "%%": %(pct)q}' 1 2 pct=100%
  {"one":"1","two":2,"%":"100%"}
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
    let mut args = args.into_iter().enumerate();
    let Some((_, format)) = args.next() else {
        return Err(USAGE.into());
    };

    let mut val = "".to_string();
    let mut last_char = None;
    let mut is_reading_named_placeholders = false;
    let mut placeholder_name: Option<String> = None;
    let mut named_placeholders = HashMap::<String, String>::new();

    for (col, ch) in format.chars().enumerate() {
        if let Some(name) = placeholder_name.as_mut() {
            // Reading a named placeholder
            match (ch, last_char) {
                ('q', Some(')')) => {
                    let value = named_placeholders.get(name).ok_or(
                        format!("no value for placeholder '%({name})q' at column {col}")
                            .as_str(),
                    )?;
                    val.push_str(&json::to_string(value)?);
                    placeholder_name = None;
                    last_char = None;
                }
                ('s', Some(')')) => {
                    let value = named_placeholders.get(name).ok_or(
                        format!("no value for placeholder '%({name})s' at column {col}")
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
        } else {
            // Reading a positional placeholder
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
fn test_format_positional() {
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
fn test_format_named() {
    let args = [
        r#"{"1": %(1)s, one: %(1)q, "true": %(true)s, "truestr": %(true)q, foo: %(foo)s, bar: %(bar)q, esc: "%%"}"#,
        "1=1",
        "true=true",
        "foo=foo",
        "bar=bar",
    ]
    .into_iter()
    .map(Into::into);

    assert_eq!(
        format(args).unwrap(),
        r#"{"1":1,"one":"1","true":true,"truestr":"true","foo":"foo","bar":"bar","esc":"%"}"#
    );
}

#[test]
fn test_format_both() {
    let args = [r#"{positional: %q, named: %(named)s}"#, "foo", "named=bar"]
        .into_iter()
        .map(Into::into);

    assert_eq!(
        format(args).unwrap(),
        r#"{"positional":"foo","named":"bar"}"#
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
        "jf: too many positional values, not enough positional placeholders"
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

#[test]
fn test_no_value_for_placeholder_name_error() {
    let args = ["%(foo)q", "bar=bar"].into_iter().map(Into::into);

    assert_eq!(
        format(args).unwrap_err().to_string(),
        "jf: no value for placeholder '%(foo)q' at column 6"
    );

    let args = ["%(foo)s", "bar=bar"].into_iter().map(Into::into);

    assert_eq!(
        format(args).unwrap_err().to_string(),
        "jf: no value for placeholder '%(foo)s' at column 6"
    );
}

#[test]
fn test_invalid_character_in_placeholder_name_error() {
    for ch in [' ', '\t', '\n', '\r', '\0', '\'', '"', '{', '}'].iter() {
        let args = [format!("%(foo{ch}bar)s)")].into_iter().map(Into::into);
        assert_eq!(
            format(args.clone()).unwrap_err().to_string(),
            format!("jf: invalid character {ch:?} in placeholder name at column 5, use numbers, letters and underscores only")
        );
    }
}

#[test]
fn test_positional_placeholder_was_used_as_named_placeholder_error() {
    let args = ["{foo: %(foo)q, bar: %q}", "foo=foo"]
        .into_iter()
        .map(Into::into);

    assert_eq!(
        format(args).unwrap_err().to_string(),
        "jf: positional placeholder '%q' at column 21 was used after named placeholders, use named placeholder syntax '%(NAME)q' instead"
    );

    let args = ["{foo: %(foo)s, bar: %s}", "foo=foo"]
        .into_iter()
        .map(Into::into);

    assert_eq!(
        format(args).unwrap_err().to_string(),
        "jf: positional placeholder '%s' at column 21 was used after named placeholders, use named placeholder syntax '%(NAME)s' instead"
    );
}

#[test]
fn test_invalid_syntax_for_value_of_named_placeholder_error() {
    let args = ["{foo: %(foo)q}", "foo"].into_iter().map(Into::into);

    assert_eq!(
        format(args).unwrap_err().to_string(),
        "jf: invalid syntax for value no. 1, use 'NAME=VALUE' syntax"
    );
}

#[test]
fn test_invalid_named_placeholder_error() {
    let args = ["%(foo)x"].into_iter().map(Into::into);
    assert_eq!(
        format(args.clone()).unwrap_err().to_string(),
        format!("jf: invalid named placeholder '%(foo)x' at column 6, use '%(foo)q' for quoted strings and '%(foo)s' for other values")
    );
}

#[test]
fn test_print_version() {
    let arg = ["jf v%v"].into_iter().map(Into::into);
    assert_eq!(format(arg).unwrap().to_string(), r#""jf v0.2.2""#);

    let args = ["{foo: %q, bar: %(bar)q, version: %v}", "foo", "bar=bar"]
        .into_iter()
        .map(Into::into);

    assert_eq!(
        format(args).unwrap().to_string(),
        r#"{"foo":"foo","bar":"bar","version":"0.2.2"}"#
    );
}
