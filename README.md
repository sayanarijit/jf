# jf

[![Crate Status](https://img.shields.io/crates/v/jf.svg)](https://crates.io/crates/jf)

[![Packaging status][repos]][repology]

[jf][jf] is a [jo][jo] alternative, A small utility to safely format and print [JSON][json] objects in the commandline.

However, unlike `jo`, where you build the JSON object by nesting `jo` outputs,
`jf` works similar to `printf`, i.e. it expects the template in [YAML][yaml] format as the first argument, and then the values for the placeholders as subsequent arguments.

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

- `%q` quoted and safely escaped JSON string
- `%s` JSON values other than string
- `%v` the `jf` version number
- `%%` a literal `%` character

And [VALUE]... [NAME=VALUE]... are the values for the placeholders.

### SYNTAX

- `%s` `%q` posiitonal placeholder
- `%(NAME)s` `%(NAME)q` named placeholder
- `%(NAME=DEFAULT)s` `%(NAME=DEFAULT)q` placeholder with default value
- `%(NAME)?s` `%(NAME)?q` placeholder with optional value
- `%*s` `%*q` expand positional values as array items
- `%**s` `%**q` expand positional values as key value pairs
- `%(NAME)*s` `%(NAME)*q` expand named values as array items
- `%(NAME)**s` `%(NAME)**q` expand named values as key value pairs

### RULES

- Pass values for positional placeholders in the same order as in the template.
- Pass values for named placeholders using `NAME=VALUE` syntax.
- Pass values for named array items using `NAME=ITEM_N` syntax.
- Pass values for named key value pairs using `NAME=KEY_N NAME=VALUE_N` syntax.
- Optional placeholders default to empty string, which is considered as null.
- Do not declare or pass positional placeholders or values after named ones.
- Expandable positional placeholder should be the last placeholder in a template.

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

jf "{%q: %(value=default)q, %(bar)**q}" foo value=bar bar=biz bar=baz
# {"foo":"bar","biz":"baz"}

jf "{str or bool: %(str)?q %(bool)?s, optional: %(optional)?q}" str=true
# {"str or bool":"true","optional":null}

jf '{1: %s, two: %q, 3: %(3)s, four: %(four=4)q, "%%": %(pct)?q}' 1 2 3=3
# {"1":1,"two":"2","3":3,"four":"4","%":null}
```

### USEFUL ALIASES

```bash
alias str='jf %q'
alias arr='jf "[%*s]"'
alias obj='jf "{%**s}"'
```

### ALIAS USAGE

```bash
str 1
# "1"

arr 1 2 3
# [1,2,3]

obj one 1 two 2 three 3
# {"one":1,"two":2,"three":3}

obj 1 2 3 $(arr 4 $(str 5))
# {"1":2,"3":[4,"5"]}
```

### RUST LIBRARY

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
