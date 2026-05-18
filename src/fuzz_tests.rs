use crate as jf;
use proptest::prelude::*;
use std::borrow::Cow;
use std::io;

fn strip_stdin_placeholders(input: String) -> String {
    input.replace("%-", "%").replace("@-", "=default")
}

fn into_stdin(lines: Vec<String>) -> Vec<io::Result<Vec<u8>>> {
    lines.into_iter()
        .map(|line| Ok(line.into_bytes()))
        .collect()
}

fn into_named_args(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .enumerate()
        .map(|(index, value)| format!("name{index}={value}"))
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn fuzz_format(template in "\\PC*", args in prop::collection::vec("\\PC*", 0..10)) {
        let input: Vec<Cow<str>> = std::iter::once(strip_stdin_placeholders(template))
            .chain(args.into_iter().map(strip_stdin_placeholders))
            .map(Cow::from)
            .collect();

        // Ensure it doesn't panic
        let _ = jf::format(input);
    }

    #[test]
    fn fuzz_render(template in "\\PC*", args in prop::collection::vec("\\PC*", 0..10)) {
        let input: Vec<Cow<str>> = std::iter::once(strip_stdin_placeholders(template))
            .chain(args.into_iter().map(strip_stdin_placeholders))
            .map(Cow::from)
            .collect();

        // Ensure it doesn't panic
        let _ = jf::render(input);
    }

    #[test]
    fn fuzz_placeholder_heavy(
        placeholders in prop::collection::vec(
            prop_oneof![
                Just("%s"), Just("%q"), Just("%%"),
                Just("%-s"), Just("%-q"), Just("%*s"), Just("%*q"),
                Just("%**s"), Just("%**q"),
                Just("%(name)s"), Just("%(name)q"), Just("%(name=def)s"),
                Just("%(name@file)q"), Just("%(name?)s"), Just("%(name)?q"),
                Just("%(name)*s"), Just("%(name)**q"),
            ],
            0..10
        ),
        garbage in prop::collection::vec("\\PC*", 0..10),
        args in prop::collection::vec("\\PC*", 0..20)
    ) {
        let mut template = String::new();
        for (p, g) in placeholders.iter().zip(garbage.iter().chain(std::iter::repeat(&"".to_string()))) {
            template.push_str(p);
            template.push_str(g);
        }
        let template = strip_stdin_placeholders(template);

        let input: Vec<Cow<str>> = std::iter::once(template)
            .chain(args.into_iter().map(|s| {
                if s.contains('=') || s.contains('@') {
                    strip_stdin_placeholders(s)
                } else {
                    format!("name={}", strip_stdin_placeholders(s))
                }
            }))
            .map(Cow::from)
            .collect();

        let _ = jf::format(input);
    }

    #[test]
    fn fuzz_render_with_finite_stdin(
        template in "\\PC*",
        args in prop::collection::vec("\\PC*", 0..10),
        stdin in prop::collection::vec("\\PC*", 0..10)
    ) {
        let input: Vec<Cow<str>> = std::iter::once(template)
            .chain(args)
            .map(Cow::from)
            .collect();

        let _ = jf::render_with_stdin(input, into_stdin(stdin));
    }

    #[test]
    fn fuzz_stdin_placeholders_do_not_block(
        placeholders in prop::collection::vec(
            prop_oneof![
                Just("%-s"), Just("%-q"), Just("%*-s"), Just("%*-q"),
                Just("%**-s"), Just("%**-q"), Just("%(name0@-)s"), Just("%(name0@-)q"),
                Just("%(name0)*s"), Just("%(name0)**q"), Just("%s"), Just("%q"), Just("%%"),
            ],
            0..12
        ),
        separators in prop::collection::vec("[,: {}\\[\\]\"a-zA-Z0-9_-]{0,4}", 0..12),
        positional_args in prop::collection::vec("\\PC*", 0..12),
        named_values in prop::collection::vec("\\PC*", 0..6),
        stdin in prop::collection::vec("\\PC*", 0..12)
    ) {
        let mut template = String::new();
        for (index, placeholder) in placeholders.iter().enumerate() {
            template.push_str(placeholder);
            if let Some(separator) = separators.get(index) {
                template.push_str(separator);
            }
        }

        let input: Vec<Cow<str>> = std::iter::once(template)
            .chain(positional_args)
            .chain(into_named_args(named_values))
            .map(Cow::from)
            .collect();

        let _ = jf::render_with_stdin(input, into_stdin(stdin));
    }

    #[test]
    fn fuzz_format_with_finite_stdin(
        positional_args in prop::collection::vec("\\PC*", 0..8),
        stdin in prop::collection::vec("\\PC*", 0..8)
    ) {
        let template = r#"{raw: %-q, positional: %q, stdin_items: [%*-q], arg_items: [%*q]}"#.to_string();
        let input: Vec<Cow<str>> = std::iter::once(template)
            .chain(positional_args)
            .map(Cow::from)
            .collect();

        let _ = jf::format_with_stdin(input, into_stdin(stdin));
    }
}
