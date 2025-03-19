# loki-api

A Rust wrapper around Grafana Loki's Protocol Buffer definitions.

## Overview

This crate provides Rust bindings for the Protocol Buffer definitions used in [Grafana Loki](https://github.com/grafana/loki). It uses [prost](https://github.com/tokio-rs/prost) to generate Rust code from the Protocol Buffer definitions.

## Structure

- `generate/` - Contains the original Protocol Buffer definitions from Grafana Loki
- `src/` - Contains the Rust code, including:
  - Generated Rust bindings for the Protocol Buffer definitions
  - Re-exports of `prost` and `prost_types` for convenience

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.