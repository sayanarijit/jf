# jf

jf "jf: %q" "JSON Format"

[![Crate Status](https://img.shields.io/crates/v/jf.svg)](https://crates.io/crates/jf)

[jf][jf] is a [jo][jo] alternative to help safely format and print [JSON][json] objects in the commandline.

However, unlike `jo`, where you build the JSON object by nesting `jo` outputs,
`jf` works similar to `printf`, i.e. it expects the template in [YAML][yaml] format as the first argument, and then the values for the placeholders as subsequent arguments.

## Usage

```bash
jf TEMPLATE [VALUE]...
```

Where TEMPLATE may contain the following placeholders:

- `%q`: Placeholder for quoted and safely escaped JSON string.
- `%s`: Placeholder for JSON values other than string.
- `%%`: Placeholder for a single `%` (i.e. escaped `%`).

And [VALUE]... are the values for the placeholders.

Use `%%` to escape a literal `%` character.

Example:

```bash
jf "hello: %q" "world"
# jo hello=world
# {"hello":"world"}

jf "hello: {beautiful: %q}" "world"
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
