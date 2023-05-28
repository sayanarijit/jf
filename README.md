# jf

[![Crate Status](https://img.shields.io/crates/v/jf.svg)](https://crates.io/crates/jf)

[![Packaging status][repos]][repology]

[jf][jf] is a [jo][jo] alternative, A small utility to safely format and print [JSON][json] objects in the commandline.

However, unlike `jo`, where you build the JSON object by nesting `jo` outputs,
`jf` works similar to `printf`, i.e. it expects the template in [YAML][yaml] format as the first argument, and then the values for the placeholders as subsequent arguments.

For example:

```bash
jf "{one: %s, two: %q, three: [%(four)s, %(five=5)q]}" 1 2 four=4
# {"one":1,"two":"2","three":[4,"5"]}
```

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
  jf [OPTION]... [--] TEMPLATE [VALUE]... [NAME=VALUE]... [NAME@FILE]...
```

### OPTIONS

| option        | help                                               |
| ------------- | -------------------------------------------------- |
| -             | alias for `-f -`, i.e. read template from stdin    |
| --            | stop parsing CLI options                           |
| -r, --raw     | print the raw rendered value without formatting    |
| -p, --pretty  | pretty print the JSON formatted output             |
| -y, --yaml    | print the output as YAML instead of JSON           |
| -h, --help    | print this help message                            |
| -v, --version | print the version number                           |
| -f, --file    | treat the template argument as a file to read from |

### TEMPLATE

Template should render into valid YAML. It can contain the following placeholders:

- `%%` a literal `%` character
- `%s` `%q` read positional argument
- `%-s` `%-q` read stdin
- `%(NAME)s` `%(NAME)q` read named value from argument
- `%(NAME=DEFAULT)s` `%(NAME=DEFAULT)q` placeholder with default value
- `%(NAME@FILE)s` `%(NAME@FILE)q` read default value from file path
- `%(NAME@-)s` `%(NAME@-)q` read default value from stdin
- `%(NAME?)s` `%(NAME?)q` nullable placeholder that defaults to null
- `%(NAME)?s` `%(NAME)?q` optional placeholder that defaults to blank
- `%*s` `%*q` expand positional args as array items
- `%*-s` `%*-q` expand stdin as array items
- `%**s` `%**q` expand positional args as key value pairs
- `%**-s` `%**-q` expand stdin as key value pairs
- `%(NAME)*s` `%(NAME)*q` expand named args as array items
- `%(NAME)**s` `%(NAME)**q` expand named args as key value pairs

Use placeholders with suffix `q` for safely quoted JSON string and `s` for JSON values
other than string.

### RULES

- Pass values for positional placeholders in the same order as in the template.
- Pass values for named placeholders using `NAME=VALUE` syntax.
- Pass values for named array items using `NAME=ITEM_N` syntax.
- Pass values for named key value pairs using `NAME=KEY_N NAME=VALUE_N` syntax.
- Pass values to stdin following the order and separate them with null byte (`\0`).
- Use `NAME@FILE` syntax to read from file where FILE can be `-` for stdin.
- Do not pass positional values after named values.
- To allow merging arrays and objects via expansion, trailing comma after `s` and `q`,
  if any, will be auto removed if no value is passed for the expandable placeholder.

### EXAMPLES

```bash
jf %s 1
# 1

jf %q 1
# "1"

jf '{%**q}' one 1 two 2 three 3
# {"one":"1","two":"2","three":"3"}

seq 1 3 | xargs printf '%s\0' | jf '[%*-s]'
# [1,2,3]

jf "{%q: %(value=default)q, %(bar)**q}" foo value=bar bar=biz bar=baz
# {"foo":"bar","biz":"baz"}

jf "{str or bool: %(str)?q %(bool)?s, nullable: %(nullable?)q}" str=true
# {"str or bool":"true","nullable":null}

jf '{1: %s, two: %q, 3: %(3)s, four: %(four=4)q, "%%": %(pct?)q}' 1 2 3=3
# {"1":1,"two":"2","3":3,"four":"4","%":null}
```

### SHELL ALIASES

You can set the following aliases in your shell:

```bash
- alias str='jf %q'
- alias arr='jf "[%*s]"'
- alias obj='jf "{%**s}"'
```

Then you can use them like this:

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
