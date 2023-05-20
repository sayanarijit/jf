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

```bash
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
- `%*s`, `%*q` for variable number of array items.
- `%**s`, `%**q` for variable number of key value pairs.

### RULES

- Pass values for positional placeholders in the same order as in the template.
- Pass values for named placeholders using `NAME=VALUE` syntax.
- Do not declare or pass positional placeholders or values after named ones.
- Nesting placeholders is prohibited.
- Variable length placeholder should be the last placeholder in a template.

### EXAMPLES

```bash
jf %s 1
# 1

jf %q 1
# "1"

jf [%*s] 1 2 3
# [1,2,3]

jf {%**q} one 1 two 2 three 3
# {"one":"1","two":"2","three":"3"}

jf "%q: %(value=default)q" foo value=bar
# {"foo":"bar"}

jf "{str_or_bool: %?(str)q %?(bool)s, optional: %?(optional)q}" str=true
# {"str_or_bool":"true","optional":null}

jf '{1: %s, two: %q, 3: %(3)s, four: %(four=4)q, "%%": %(pct)q}' 1 2 3=3 pct=100%
# {"1":1,"two":"2","3":3,"four":"4","%":"100%"}
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
