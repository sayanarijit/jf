# jf

jf %q "JSON Formatted"

[![Crate Status](https://img.shields.io/crates/v/jf.svg)](https://crates.io/crates/jf)

[jf][jf] is a [jo][jo] alternative to help safely format and print [JSON][json] objects in the commandline.

However, unlike `jo`, where you build the JSON object by nesting `jo` outputs,
`jf` works similar to `printf`, i.e. it expects the template in [YAML][yaml] format as the first argument, and then the values for the placeholders as subsequent arguments.

## Install

```bash
cargo install jf
```

Or [download binary from the latest release][bins].

## Usage

```bash
jf TEMPLATE [VALUE]... [NAME=VALUE]...
```

Where TEMPLATE may contain the following placeholders:

- `%q` or `%(NAME)q`: For quoted and safely escaped JSON string.
- `%s` or `%(NAME)s`: For JSON values other than string.

And [VALUE]... [[NAME=]VALUE]... are the values for the placeholders.

- Use `%s` or `%q` syntax to pass positional values.
- Use `%(NAME)s` or `%(NAME)q` syntax to pass named values.
- Use `%%` to escape a literal `%` character.
- Pass the values for named placeholders using `NAME=VALUE` syntax.
- Do not use positional placeholders after named placeholders.

Example:

```bash
jf "hello: %q" world
# jo hello=world
# {"hello":"world"}

jf "hello: {beautiful: %(what)q}" what=world
# jo hello=$(jo beautiful=world)
# {"hello":{"beautiful":"world"}}

jf "d: {m: %s, n: %s}" 10 20
# jo d[m]=10 d[n]=20
# {"d":{"m":10,"n":20}}

jf "{a: {b: %s, c: {d: %s, f: %s}, d: {e: [%s, %q]}}, b: {e: [%q]}}" 0 1 true 2 sam hi
# jo -d\|first_char_only a\|b=0 a\|c\|d=1 a\|d\|e[]=2 a\|d\|e[]=sam a\|c[f]@1 b\|e[]g=hi
# {"a":{"b":0,"c":{"d":1,"f":true},"d":{"e":[2,"sam"]}},"b":{"e":["hi"]}}
```

[jf]: https://github.com/sayanarijit/jf
[jo]: https://github.com/jpmens/jo
[yaml]: https://yaml.org
[json]: https://json.org
[bins]: https://github.com/sayanarijit/jf/releases/latest
