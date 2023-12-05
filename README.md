# Rust in chaiNNer

[![PyPI](https://img.shields.io/pypi/v/chainner_ext)](https://pypi.org/project/chainner-ext/)

This repository contains code to implement nodes for [chaiNNer](https://github.com/chaiNNer-org/chaiNNer) in Rust.

The main goals of this project are as follows:

1. Provide high-performance implementations of algorithms that are not possible to implement efficiently in Python.
1. Wrap existing Rust libraries.
1. Provide cross-platform Python bindings for all of the above.

In essence, this project bridges the performance and compatibility gaps caused by Python.

## Contributing

Install Rust and Python onto your machine. We recommend VSCode as the IDE of choice for writing Rust code.

Useful commands:

- `cargo check` - check that the code compiles.
- `cargo clippy` - check that the code compiles and passes numerous lints.
- `cargo test` - run all tests.
- `cargo test --workspace --features snap-write` - run all tests and overwrite outdated snapshots.
- `cargo bench` - run all benchmarks.

### Structure

This project consists of multiple crates:

- `bindings` - Python bindings for the other crates.
- `image-core` - The core interfaces used by the other crates.
- `image-ops` - Implementations for image operations.
- `regex-py` - Implementation for the regex API exposed by `bindings`.
- `test-util` - Utilities for testing.

### Testing the bindings

```bash
python install-locally.py
```

This will build the bindings and install them into the current Python environment. Start chaiNNer using `npm run dev` and it should use the new bindings.
