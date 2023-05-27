use crate as jf;
use serde_json as json;
use std::borrow::Cow;
use std::io;

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
    .map(Into::into);

    assert_eq!(
        jf::format(args).unwrap(),
        r#"{"1":1,"one":"1","true":true,"truestr":"true","foo":"foo","bar":"bar","esc":"%"}"#
    );
}

#[test]
fn test_format_from_stdin() {
    let mut chars = r#"{%q: %-s, %q: %-s, %q: %-s}"#.chars().enumerate();

    let mut stdin = ["1", "2", "3"]
        .map(Into::into)
        .map(io::Result::Ok)
        .into_iter()
        .enumerate();

    let mut args = ["one", "two", "three"]
        .map(Cow::from)
        .into_iter()
        .enumerate();

    let (res, _) = jf::format_partial(&mut chars, &mut args, &mut stdin).unwrap();
    assert_eq!(res, r#"{"one": 1, "two": 2, "three": 3}"#);

    let mut chars =
        r#"{"1": %-s, one: %q, "true": %s, truestr: %-q, foo: %-s, bar: %q, esc: "%%"}"#
            .chars()
            .enumerate();

    let mut stdin = ["1", "true", "foo"]
        .map(Into::into)
        .map(io::Result::Ok)
        .into_iter()
        .enumerate();

    let mut args = ["1", "true", "bar"].map(Cow::from).into_iter().enumerate();

    let (res, _) = jf::format_partial(&mut chars, &mut args, &mut stdin).unwrap();
    assert_eq!(
        res,
        r#"{"1": 1, one: "1", "true": true, truestr: "true", foo: foo, bar: "bar", esc: "%"}"#
    );
}

#[test]
fn test_format_expand_items_from_stdin() {
    let mut chars = r#"[start, %*-s, mid, %*s, end]"#.chars().enumerate();

    let mut stdin = ["1", "true", "foo"]
        .map(Into::into)
        .map(io::Result::Ok)
        .into_iter()
        .enumerate();

    let mut args = ["2", "false", "bar"].map(Cow::from).into_iter().enumerate();

    let (res, _) = jf::format_partial(&mut chars, &mut args, &mut stdin).unwrap();
    assert_eq!(res, r#"[start, 1,true,foo, mid, 2,false,bar, end]"#);
}

#[test]
fn test_format_expand_pairs_from_stdin() {
    let mut chars = r#"{args: {%**q}, stdin: {%**-q}}"#.chars().enumerate();

    let mut stdin = ["one", "1", "two", "2"]
        .map(Into::into)
        .map(io::Result::Ok)
        .into_iter()
        .enumerate();

    let mut args = ["three", "3"].map(Cow::from).into_iter().enumerate();

    let (res, _) = jf::format_partial(&mut chars, &mut args, &mut stdin).unwrap();
    assert_eq!(
        res,
        r#"{args: {"three":"3"}, stdin: {"one":"1","two":"2"}}"#
    );
}

#[test]
fn test_format_merge_arrays() {
    let args = ["[%(a)*s, %(b)*s]"].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), "[]");

    let args = ["[%(a)*s, %(b)*s]", "a=1", "a=2"].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), "[1,2]");

    let args = ["[%(a)*s, %(b)*s]", "b=1", "b=2"].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), "[1,2]");

    let args = ["[%(a)*s, %(b)*s]", "a=1", "b=2", "a=3", "b=4"].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), "[1,3,2,4]");
}

#[test]
fn test_format_merge_objs() {
    let args = ["{%(a)*s, %(b)*s}"].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), "{}");

    let args = ["{%(a)**s, %(b)**s}", "a=1", "a=2"].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), r#"{"1":2}"#);

    let args = ["{%(a)**s, %(b)**s}", "b=1", "b=2"].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), r#"{"1":2}"#);

    let args = ["{%(a)**s, %(b)**s}", "a=1", "b=2", "a=3", "b=4"].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), r#"{"1":3,"2":4}"#);
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
    .map(Into::into);

    assert_eq!(
        jf::format(args).unwrap(),
        r#"{"1":1,"one":"1","true":true,"truestr":"true","foo":"foo","bar":"bar","esc":"%"}"#
    );

    assert_eq!(
        jf::format([r#"{1: %(1=1)s, one: %(1=one)q}"#].map(Into::into)).unwrap(),
        r#"{"1":1,"one":"one"}"#
    )
}

#[test]
fn test_format_both() {
    let args =
        [r#"{positional: %q, named: %(named)s}"#, "foo", "named=bar"].map(Into::into);

    assert_eq!(
        jf::format(args).unwrap(),
        r#"{"positional":"foo","named":"bar"}"#
    );
}

#[test]
fn test_format_named_from_file_path() {
    let args = ["%(NAME)q", "NAME@./src/usage.txt"].map(Into::into);

    assert_eq!(
        jf::format(args).unwrap(),
        json::to_string(jf::USAGE).unwrap()
    );
}

#[test]
fn test_format_named_from_stdin() {
    let mut chars = "{%(FOO)q: %(BAR)q}".chars().enumerate();
    let mut stdin = ["foo", "bar"]
        .map(Into::into)
        .map(io::Result::Ok)
        .into_iter()
        .enumerate();
    let mut args = ["FOO@-", "BAR@-"].map(Cow::from).into_iter().enumerate();

    let (res, _) = jf::format_partial(&mut chars, &mut args, &mut stdin).unwrap();

    assert_eq!(res, r#"{"foo": "bar"}"#);
}

#[test]
fn test_format_named_with_default() {
    let args = [
        r#"{"1": %(1=1)s, one: %(1=1)q, foo: %(foo=default)q, empty: %(bar=)q, esc: %(x=(\))q, multi=: %(a=b=c)q}"#,
        "foo=bar",
    ]
    .map(Into::into);
    assert_eq!(
        jf::format(args).unwrap(),
        r#"{"1":1,"one":"1","foo":"bar","empty":"","esc":"()","multi=":"b=c"}"#
    );
}

#[test]
fn test_format_named_with_default_from_file() {
    let args = ["%(foo@./src/usage.txt)q"].map(Into::into);
    assert_eq!(
        jf::format(args).unwrap(),
        json::to_string(jf::USAGE).unwrap()
    );

    let args = ["%(foo@./src/usage.txt)q", "foo=bar"].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), r#""bar""#);
}

#[test]
fn test_format_named_with_default_from_stdin() {
    let mut chars = "%(foo@-)q".chars().enumerate();
    let mut args = [].into_iter().enumerate();
    let mut stdin = ["foo"]
        .map(Into::into)
        .map(io::Result::Ok)
        .into_iter()
        .enumerate();

    let (res, _) = jf::format_partial(&mut chars, &mut args, &mut stdin).unwrap();
    assert_eq!(res, r#""foo""#);

    let mut chars = "%(foo@-)q".chars().enumerate();
    let mut args = ["foo=bar"].map(Into::into).into_iter().enumerate();

    let mut stdin = ["foo"]
        .map(Into::into)
        .map(io::Result::Ok)
        .into_iter()
        .enumerate();
    let (res, _) = jf::format_partial(&mut chars, &mut args, &mut stdin).unwrap();
    assert_eq!(res, r#""bar""#);
}

#[test]
fn test_unexpected_eof() {
    let mut chars = "%(foo@-)q".chars().enumerate();
    let mut args = [].into_iter().enumerate();
    let mut stdin = [].into_iter().enumerate();

    let err = jf::format_partial(&mut chars, &mut args, &mut stdin)
        .unwrap_err()
        .to_string();

    assert_eq!(err, "io: unexpected end of input");
}

#[test]
fn test_format_optional() {
    let args = [r#"{foo: %(foo)?q, bar: %(bar)?q}"#, "foo=foo"].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), r#"{"foo":"foo","bar":null}"#);

    let args = [r#"{foo: %(foo)?q, bar: %(bar)?q}"#, "bar=bar"].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), r#"{"foo":null,"bar":"bar"}"#);

    let args = [
        r#"{"null": %(1)?s %(one)?q, "2": %(2)?s %(two)?q, three: %(3)?s %(three)?q}"#,
        "2=2",
        "three=3",
    ]
    .map(Into::into);

    assert_eq!(
        jf::format(args).unwrap(),
        r#"{"null":null,"2":2,"three":"3"}"#
    );
}

#[test]
fn test_format_expand_positional_items() {
    let args = [r#"[%*s]"#].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), r#"[]"#);

    let args = [r#"{foo: [1, %*s, 4]}"#, "2", "3"].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), r#"{"foo":[1,2,3,4]}"#);

    let args = [r#"{foo: [1, %*q, 4]}"#, "2", "3"].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), r#"{"foo":[1,"2","3",4]}"#);
}

#[test]
fn test_format_expand_positional_pairs() {
    let args = [r#"{%**s}"#].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), r#"{}"#);

    let args = [r#"{foo: bar, %**s, 2: 2}"#, "1", "1"].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), r#"{"foo":"bar","1":1,"2":2}"#);

    let args = [r#"{foo: {%**q, 3: 3}}"#, "one", "1", "two", "2"].map(Into::into);
    assert_eq!(
        jf::format(args).unwrap(),
        r#"{"foo":{"one":"1","two":"2","3":3}}"#
    );
}

#[test]
fn test_format_named_items() {
    let args = [r#"[%(foo)*s]"#].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), r#"[]"#);

    let args = [
        r#"[%(foo)s, %(bar)q, %(foo)*s, %(bar)*q]"#,
        "foo=1",
        "foo=2",
        "bar=3",
        "bar=4",
    ]
    .map(Into::into);
    assert_eq!(jf::format(args).unwrap(), r#"[1,"3",1,2,"3","4"]"#);
}

#[test]
fn test_format_named_pairs() {
    let args = [r#"{%(foo)**s}"#].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), r#"{}"#);

    let args = [
        r#"{foo: %(foo)s, bar: %(bar)q, %(foo)**s, %(bar)**q}"#,
        "foo=one",
        "foo=1",
        "foo=two",
        "foo=2",
        "bar=three",
        "bar=3",
        "bar=four",
        "bar=4",
    ]
    .map(Into::into);
    assert_eq!(
        jf::format(args).unwrap(),
        r#"{"foo":"one","bar":"three","one":1,"two":2,"three":"3","four":"4"}"#
    );
}

#[test]
fn test_render() {
    let args = ["%*s", "1", "2", "3"].map(Into::into);
    assert_eq!(jf::render(args).unwrap(), "1,2,3");

    let args = ["%s   %q, (%s)", "1", "2", "3"].map(Into::into);
    assert_eq!(jf::render(args).unwrap(), r#"1   "2", (3)"#);
}

#[test]
fn test_yaml() {
    let args = ["{a: b, c: d, e: [f, g]}"].map(Into::into);
    assert_eq!(jf::format_yaml(args).unwrap(), "a: b\nc: d\ne:\n- f\n- g\n");
}

#[test]
fn test_pretty_json() {
    let args = ["{a: b, c: d, e: [f, g]}"].map(Into::into);
    assert_eq!(
        jf::format_pretty(args).unwrap(),
        "{\n  \"a\": \"b\",\n  \"c\": \"d\",\n  \"e\": [\n    \"f\",\n    \"g\"\n  ]\n}"
    );
}

#[test]
fn test_optional_placeholder_with_default_value_error() {
    let args = [r#"%(foo=bar)?q"#].map(Into::into);

    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "jf: optional placeholder 'foo' at column 10 cannot have a default value"
    );
}

#[test]
fn test_nullable_placeholder_must_end_with_error() {
    let args = [r#"%(foo?bar)q"#].map(Into::into);

    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "jf: nullable placeholder 'foo' at column 5 must end with '?)'"
    );
}

#[test]
fn test_named_expandable_placeholder_with_default_value_error() {
    let args = [r#"%(foo=default)*q"#].map(Into::into);

    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "jf: expandable placeholder 'foo' at column 15 cannot have a default value"
    );

    let args = [r#"%(foo=default)**q"#].map(Into::into);
    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "jf: expandable placeholder 'foo' at column 16 cannot have a default value"
    );
}

#[test]
fn test_missing_name_error() {
    let args = [r#"%()s"#].map(Into::into);
    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "jf: placeholder missing name at column 3"
    );

    let args = [r#"%(=foo)q"#].map(Into::into);
    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "jf: placeholder missing name at column 7"
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
    .map(Into::into);

    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "jf: placeholder missing value at column 61"
    );

    let args = [r#"{%**q}"#, "1"].map(Into::into);

    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "jf: placeholder missing value at column 4"
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
    .map(Into::into);

    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "jf: too many positional values, not enough positional placeholders"
    );
}

#[test]
fn test_invalid_placeholder_error() {
    let args = ["%z"].map(Into::into);
    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "jf: invalid placeholder '%z' at column 1, use one of '%s' or '%q', or escape it using '%%'"
    );

    let args = ["%*z"].map(Into::into);
    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "jf: invalid placeholder '%*z' at column 2, use one of '%*s' or '%*q', or escape it using '%%'"
    );

    let args = ["%**z"].map(Into::into);
    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "jf: invalid placeholder '%**z' at column 3, use one of '%**s' or '%**q', or escape it using '%%'"
    );
}

#[test]
fn test_incomplete_placeholder_error() {
    for arg in [
        "%",
        "%-",
        "%(",
        "%()",
        "%(foo",
        "%(foo)",
        "%(foo)?",
        "%(foo)*",
        "%(foo)**",
        "%(foo=",
        "%(foo=bar",
        "%(foo@README.md",
        "%(foo=bar)",
        "%(foo@README.md)",
        "%(foo=bar)*",
        "%(foo@README.md)*",
        "%(foo=bar)**",
        "%(foo@README.md)**",
    ] {
        assert_eq!(
            jf::format([arg].map(Into::into)).unwrap_err().to_string(),
            "jf: template ended with incomplete placeholder"
        );
    }
}

#[test]
fn test_not_enough_arguments_error() {
    let usage_err = jf::format([]).unwrap_err().to_string();
    assert!(usage_err.contains("not enough arguments"));
}

#[test]
fn test_yaml_error() {
    let args = ["{}{}"].map(Into::into);

    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "yaml: deserializing from YAML containing more than one document is not supported",
    );
}

#[test]
fn test_json_error() {
    let args = ["{null: null}"].map(Into::into);

    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "json: key must be a string",
    );
}

#[test]
fn test_io_error() {
    let args = ["%(devnull@/usr/bin/env)q"].map(Into::into);

    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "io: stream did not contain valid UTF-8",
    )
}

#[test]
fn test_no_value_for_placeholder_name_error() {
    let args = ["%(foo)q", "bar=bar"].map(Into::into);

    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "jf: no value for placeholder '%(foo)q' at column 6"
    );

    let args = ["%(foo)s", "bar=bar"].map(Into::into);

    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "jf: no value for placeholder '%(foo)s' at column 6"
    );

    let args = ["%(foo=default)q: %(foo)q"].map(Into::into);

    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "jf: no value for placeholder '%(foo)q' at column 23"
    );

    let args = ["%(foo=default)q: %(bar)s"].map(Into::into);

    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "jf: no value for placeholder '%(bar)s' at column 23"
    );
}

#[test]
fn test_invalid_character_in_placeholder_name_error() {
    for ch in [' ', '\t', '\n', '\r', '\0', '\'', '"', '{', '}'].iter() {
        let args = [format!("%(foo{ch}bar)s)")].map(Into::into);
        assert_eq!(
            jf::format(args.clone()).unwrap_err().to_string(),
            format!("jf: invalid character {ch:?} in placeholder name at column 5, use numbers, letters and underscores only")
        );
    }
}

#[test]
fn test_positional_placeholder_was_used_as_named_placeholder_error() {
    let args = ["{foo: %(foo)q, bar: %q}", "foo=foo"].map(Into::into);

    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "jf: positional placeholder '%q' at column 21 was used after named placeholders, use named placeholder syntax '%(NAME)q' instead"
    );

    let args = ["{foo: %(foo)s, bar: %s}", "foo=foo"].map(Into::into);

    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "jf: positional placeholder '%s' at column 21 was used after named placeholders, use named placeholder syntax '%(NAME)s' instead"
    );
}

#[test]
fn test_invalid_syntax_for_value_of_named_placeholder_error() {
    let args = ["{foo: %(foo)q}", "foo"].map(Into::into);

    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "jf: invalid syntax for value no. 1, use 'NAME=VALUE' or 'NAME@FILE' syntax"
    );
}

#[test]
fn test_invalid_named_placeholder_error() {
    let args = ["%(foo)x"].map(Into::into);
    assert_eq!(
        jf::format(args.clone()).unwrap_err().to_string(),
        format!("jf: invalid named placeholder '%(foo)x' at column 6, use '%(foo)q' for quoted strings and '%(foo)s' for other values")
    );

    let args = ["%(foo)-"].map(Into::into);
    assert_eq!(
        jf::format(args.clone()).unwrap_err().to_string(),
        format!("jf: invalid named placeholder '%(foo)-' at column 6, use '%(foo)q' for quoted strings and '%(foo)s' for other values")
    );

    let args = ["%(foo)*x"].map(Into::into);
    assert_eq!(
        jf::format(args.clone()).unwrap_err().to_string(),
        format!("jf: invalid named placeholder '%(foo)*x' at column 7, use '%(foo)*q' for quoted strings and '%(foo)*s' for other values")
    );

    let args = ["%(foo)*-"].map(Into::into);
    assert_eq!(
        jf::format(args.clone()).unwrap_err().to_string(),
        format!("jf: invalid named placeholder '%(foo)*-' at column 7, use '%(foo)*q' for quoted strings and '%(foo)*s' for other values")
    );

    let args = ["%(foo)**x"].map(Into::into);
    assert_eq!(
        jf::format(args.clone()).unwrap_err().to_string(),
        format!("jf: invalid named placeholder '%(foo)**x' at column 8, use '%(foo)**q' for quoted strings and '%(foo)**s' for other values")
    );

    let args = ["%(foo)**-"].map(Into::into);
    assert_eq!(
        jf::format(args.clone()).unwrap_err().to_string(),
        format!("jf: invalid named placeholder '%(foo)**-' at column 8, use '%(foo)**q' for quoted strings and '%(foo)**s' for other values")
    );
}

#[test]
fn test_usage_example() {
    let args = ["%s", "1"].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), "1");

    let args = ["%q", "1"].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), r#""1""#);

    let args = ["[%*s]", "1", "2", "3"].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), "[1,2,3]");

    let args = ["{%**q}", "one", "1", "two", "2", "three", "3"].map(Into::into);
    assert_eq!(
        jf::format(args).unwrap(),
        r#"{"one":"1","two":"2","three":"3"}"#
    );

    let args = [
        "{%q: %(value=default)q, %(bar)**q}",
        "foo",
        "value=bar",
        "bar=biz",
        "bar=baz",
    ]
    .map(Into::into);
    assert_eq!(jf::format(args).unwrap(), r#"{"foo":"bar","biz":"baz"}"#);

    let args = [
        "{str or bool: %(str)?q %(bool)?s, nullable: %(nullable?)q}",
        "str=true",
    ]
    .map(Into::into);
    assert_eq!(
        jf::format(args).unwrap(),
        r#"{"str or bool":"true","nullable":null}"#
    );

    let args = [
        r#"{1: %s, two: %q, 3: %(3)s, four: %(four=4)q, "%%": %(pct?)q}"#,
        "1",
        "2",
        "3=3",
    ]
    .map(Into::into);
    assert_eq!(
        jf::format(args).unwrap(),
        r#"{"1":1,"two":"2","3":3,"four":"4","%":null}"#
    );
}

#[cfg(feature = "manpage")]
#[test]
fn update_manpage() {
    let man = std::process::Command::new("txt2man")
        .arg("-P")
        .arg("jf")
        .arg("-t")
        .arg("jf")
        .arg("-d")
        .arg("1")
        .arg("jf")
        .arg("./src/usage.txt")
        .output()
        .unwrap()
        .stdout;
    std::fs::write("assets/jf.1", man).unwrap();
}
