# jf

[![Crate Status](https://img.shields.io/crates/v/jf.svg)](https://crates.io/crates/jf)

[jf][jf] is a [jo][jo] alternative, A small utility to safely format and print [JSON][json] objects in the commandline.

However, unlike `jo`, where you build the JSON object by nesting `jo` outputs,
`jf` works similar to `printf`, i.e. it expects the template in [YAML][yaml] format as the first argument, and then the values for the placeholders as subsequent arguments.

## Install

#### [Cargo][cargo]

```bash
cargo install jf
```

#### [Nixpkgs][nixpkgs]

```bash
nix-env -f https://github.com/NixOS/nixpkgs/tarball/nixos-unstable -iA jf
```

Or [download binary from the latest release][bins].

## Usage

```bash
jf TEMPLATE [VALUE]... [NAME=VALUE]...
```

Where TEMPLATE may contain the following placeholders:

- `%q`, `%(NAME)q`, `%(NAME=DEFAULT)q` for quoted and safely escaped JSON string.
- `%s`, `%(NAME)s`, `%(NAME=DEFAULT)s` for JSON values other than string.

And [VALUE]... [NAME=VALUE]... are the values for the placeholders.

- Use `%s` or `%q` syntax to declare positional placeholders.
- Use `%(NAME)s` or `%(NAME)q` syntax to declare named placeholders.
- Use `%(NAME=DEFAULT)s` or `%(NAME=DEFAULT)q` syntax to declare default values for named placeholders.
- Use `%%` to escape a literal `%` character.
- Pass values for positional placeholders in the same order as in the template.
- Pass values for named placeholders using `NAME=VALUE` syntax.
- Do not declare or pass positional placeholders or values after named ones.
- To get the `jf` version number, run `jf %v`.

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
# jo -d\|first_char_only a\|b=0 a\|c\|d=1 a\|d\|e[]=2 a\|d\|e[]=sam a\|c[f]@1 b\|e[]=hi
# {"a":{"b":0,"c":{"d":1,"f":true},"d":{"e":[2,"sam"]}},"b":{"e":["hi"]}}
```

## Packaging

[![Packaging status][repos]][repology]

[jf]: https://github.com/sayanarijit/jf
[jo]: https://github.com/jpmens/jo
[yaml]: https://yaml.org
[json]: https://json.org
[bins]: https://github.com/sayanarijit/jf/releases/latest
[cargo]: https://crates.io/crates/jf
[nixpkgs]: https://github.com/NixOS/nixpkgs/blob/nixos-unstable/pkgs/development/tools/jf/default.nix
[repology]: https://repology.org/project/jf/versions
[repos]: https://repology.org/badge/vertical-allrepos/jf.svg
