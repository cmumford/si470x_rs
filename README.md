# si470x
[![CI](https://github.com/cmumford/si470x/actions/workflows/ci.yml/badge.svg)](https://github.com/cmumford/si470x/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A pure Rust Si470x radio IC driver.

## building

may need to:

*one time per workspace*:
```sh
rustup override set esp
```

To view:
```sh
rustup override list
```

In a shell before building:

```sh
. "$HOME/.cargo/env"
. "$HOME/export-esp.sh"
```
