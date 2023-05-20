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
fn test_format_optional() {
    let args = [r#"{foo: %?(foo)q, bar: %?(bar)q}"#, "foo=foo"].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), r#"{"foo":"foo","bar":null}"#);

    let args = [r#"{foo: %?(foo)q, bar: %?(bar)q}"#, "bar=bar"].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), r#"{"foo":null,"bar":"bar"}"#);

    let args = [
        r#"{"null": %?(1)s %?(one)q, "2": %?(2)s %?(two)q, three: %?(3)s %?(three)q}"#,
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
fn test_format_var_arr() {
    let args = [r#"{foo: [1, %*s, 4]}"#, "2", "3"].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), r#"{"foo":[1,2,3,4]}"#);

    let args = [r#"{foo: [1, %*q, 4]}"#, "2", "3"].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), r#"{"foo":[1,"2","3",4]}"#);
}

#[test]
fn test_format_var_obj() {
    let args = [r#"{foo: bar, %**s, 2: 2}"#, "1", "1"].map(Into::into);
    assert_eq!(jf::format(args).unwrap(), r#"{"foo":"bar","1":1,"2":2}"#);

    let args = [r#"{foo: {%**q, 3: 3}}"#, "one", "1", "two", "2"].map(Into::into);
    assert_eq!(
        jf::format(args).unwrap(),
        r#"{"foo":{"one":"1","two":"2","3":3}}"#
    );
}

#[test]
fn test_optional_placeholder_with_default_value_error() {
    let args = [r#"%?(foo=bar)q"#].map(Into::into);

    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "jf: optional placeholder 'foo' at column 6 cannot have a default value"
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
    let args = ["foo: %z", "bar"].map(Into::into);

    assert_eq!(
        jf::format(args).unwrap_err().to_string(),
        "jf: invalid placeholder '%z' at column 6, use one of '%s' or '%q', or escape it using '%%'"
    );
}

#[test]
fn test_incomplete_placeholder_error() {
    for arg in [
        "%",
        "%(",
        "%()",
        "%(foo",
        "%(foo)",
        "%(foo=",
        "%(foo=bar",
        "%(foo=bar)",
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
    assert!(usage_err.contains("USAGE: jf TEMPLATE [VALUE]... [NAME=VALUE]..."));
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
    for ch in [' ', '\t', '\n', '\r', '\0', '\'', '"', '{', '}', '?'].iter() {
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
        "jf: invalid syntax for value no. 1, use 'NAME=VALUE' syntax"
    );
}

#[test]
fn test_invalid_named_placeholder_error() {
    let args = ["%(foo)x"].map(Into::into);
    assert_eq!(
        jf::format(args.clone()).unwrap_err().to_string(),
        format!("jf: invalid named placeholder '%(foo)x' at column 6, use '%(foo)q' for quoted strings and '%(foo)s' for other values")
    );
}

#[test]
fn test_usage_example() {
    let args = [
        r#"{1: %s, two: %q, 3: %(3)s, four: %(four=4)q, "%%": %(pct)q}"#,
        "1",
        "2",
        "3=3",
        "pct=100%",
    ]
    .map(Into::into);

    assert_eq!(
        jf::format(args).unwrap().to_string(),
        r#"{"1":1,"two":"2","3":3,"four":"4","%":"100%"}"#
    );
}

#[test]
fn test_print_version() {
    let arg = ["jf v%v"].map(Into::into);
    assert_eq!(jf::format(arg).unwrap().to_string(), r#""jf v0.2.7""#);

    let args =
        ["{foo: %q, bar: %(bar)q, version: %v}", "foo", "bar=bar"].map(Into::into);

    assert_eq!(
        jf::format(args).unwrap().to_string(),
        r#"{"foo":"foo","bar":"bar","version":"0.2.7"}"#
    );
}
