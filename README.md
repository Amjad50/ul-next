# ul-next

[![Build](https://github.com/Amjad50/ul-next/actions/workflows/ci.yml/badge.svg)](https://github.com/Amjad50/ul-next/actions/workflows/ci.yml)
[![Crates.io ul-next](https://img.shields.io/crates/v/ul-next)](https://crates.io/crates/ul-next)
[![docs.rs (with version)](https://img.shields.io/docsrs/ul-next/latest)](https://docs.rs/ul-next)


High level rust bindings for [Ultralight]. Replacement for [`rust-ul`].

[Ultralight] is a lightweight, high-performance HTML rendering engine designed for applications that require a high degree of customization. Using GPU-accelerated HTML rendering, it's a great fit for rendering user interfaces in games, and other applications.

## Extra files

You need to include `resources` folder in the execution directory.

You can find the resources folder in the [Ultralight SDK](https://github.com/ultralight-ux/Ultralight/releases/latest)

## Examples

To see how this library is used, please check the examples in the [`examples`](./examples/) directory.

```sh
cargo run --example=basic_app
```

> For now, must be run from the root of the project, as it needs to find the `resources` folder in the `examples` directory.

## License
This project uses the `ULTRALIGHT FREE LICENSE AGREEMENT - V1`. See [LICENSE](./LICENSE.txt) for more information.


[`rust-ul`]: https://github.com/psychonautwiki/rust-ul
[Ultralight]: https://ultralig.ht