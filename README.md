# Tunnel

**Tunnel** is a library for easy sending and receiving of data over P2P.

Fundamentally, Tunnel is just a small wrapper over [iroh](https://github.com/n0-computer/iroh), a convenient P2P library.

# Getting started

Tunnel can be used as either a Rust crate, a Python library or a WASM module.

## Rust crate

The best way to learn how to use Tunnel is through the ["sending"](/examples/sending.rs) and ["receiving"](/examples/receiving.rs) examples.

Tunnel requires heavy usage of `async`. As such, it is recommended to use [Tokio](https://github.com/tokio-rs/tokio) or similar.

# License

This project is licensed under the MIT license ([LICENSE](/LICENSE) or http://opensource.org/licenses/MIT).

# Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you shall be licensed as above, without any additional terms or conditions.