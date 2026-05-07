# si470x
[![CI](https://github.com/cmumford/si470x_rs/actions/workflows/ci.yml/badge.svg)](https://github.com/cmumford/si470x_rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A driver for the Si4703 and Si4702 FM radio IC's. This library supports:

* Pure Rust
* `no-std` build environment
* Either synchronous or asynchronous API
* Platform indepdence via use of `embedded-hal`

## Examples

See `//examples` for simple working examples to initialize and interract with
a Si470x.

## Rust Setup Notes

may need to *one time per workspace*:

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

## Building

```sh
make
```

