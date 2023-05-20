# jf

[![Crate Status](https://img.shields.io/crates/v/jf.svg)](https://crates.io/crates/jf)

[jf][jf] is a [jo][jo] alternative, A small utility to safely format and print [JSON][json] objects in the commandline.

However, unlike `jo`, where you build the JSON object by nesting `jo` outputs,
`jf` works similar to `printf`, i.e. it expects the template in [YAML][yaml] format as the first argument, and then the values for the placeholders as subsequent arguments.

[![Packaging status][repos]][repology]

### INSTALL

#### [Cargo][cargo]

As a CLI tool

```bash
cargo install jf
```

Or as a library:

```bash
cargo add jf
```

#### [Nixpkgs][nixpkgs]

```bash
nix-env -f https://github.com/NixOS/nixpkgs/tarball/nixos-unstable -iA jf
```

#### [Binaries][bins]

### USAGE

```
jf TEMPLATE [VALUE]... [NAME=VALUE]...
```

Where TEMPLATE may contain the following placeholders:

- `%q` for quoted and safely escaped JSON string.
- `%s` for JSON values other than string.
- `%v` for the `jf` version number.
- `%%` for a literal `%` character.

And [VALUE]... [NAME=VALUE]... are the values for the placeholders.

### SYNTAX

- `%s`, `%q` for posiitonal placeholders.
- `%(NAME)s`, `%(NAME)q` for named placeholders.
- `%(NAME=DEFAULT)s`, `%(NAME=DEFAULT)q` for placeholders with default values.
- `%?(NAME)s`, `%?(NAME)q` for optional placeholders.

### RULES

- Pass values for positional placeholders in the same order as in the template.
- Pass values for named placeholders using `NAME=VALUE` syntax.
- Do not declare or pass positional placeholders or values after named ones.
- Nesting placeholders is prohibited.

### EXAMPLES

```bash
jf %s 1
# 1

jf %q 1
# "1"

jf "%q: %(value=default)q" foo value=bar
# {"foo":"bar"}

jf "{ str_or_bool: %?(str)q %?(bool)s, optional: %?(optional)q }" str=true
# {"str_or_bool":"true","optional":null}

jf '{1: %s, two: %q, 3: %(3)s, four: %(four=4)q, "%%": %(pct)q}' 1 2 3=3 pct=100%
# {"1":1,"two":"2","3":3,"four":"4","%":"100%"}
```

#### vs jo

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
# jo -d\|first_char_only a\|b=0 a\|c\|d=1 a\|d\|e[]=2 a\|d\|e[]=sam a\|c[f]@1 b\|e[]=hi
# {"a":{"b":0,"c":{"d":1,"f":true},"d":{"e":[2,"sam"]}},"b":{"e":["hi"]}}
```

#### Rust Library

```rust
let json = match jf::format(["%q", "JSON Formatted"].map(Into::into)) {
    Ok(value) => value,
    Err(jf::Error::Usage) => {
        bail!("usage: mytool: TEMPLATE [VALUE]... [NAME=VALUE]...")
    }
    Err(jf::Error::Jf(e)) => bail!("mytool: {e}"),
    Err(jf::Error::Json(e)) => bail!("mytool: json: {e}"),
    Err(jf::Error::Yaml(e)) => bail!("mytool: yaml: {e}"),
};
```

[jf]: https://github.com/sayanarijit/jf
[jo]: https://github.com/jpmens/jo
[yaml]: https://yaml.org
[json]: https://json.org
[bins]: https://github.com/sayanarijit/jf/releases/latest
[cargo]: https://crates.io/crates/jf
[nixpkgs]: https://github.com/NixOS/nixpkgs/blob/nixos-unstable/pkgs/development/tools/jf/default.nix
[repology]: https://repology.org/project/jf/versions
[repos]: https://repology.org/badge/vertical-allrepos/jf.svg
