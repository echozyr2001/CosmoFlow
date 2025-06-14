# Contributing to CosmoFlow

First off, thank you for considering contributing to CosmoFlow! It's people like you that make CosmoFlow such a great tool.

We welcome any type of contribution, not just code. You can help with:

*   **Reporting a bug**
*   **Discussing the current state of the code**
*   **Submitting a fix**
*   **Proposing new features**
*   **Becoming a maintainer**

## Getting Started

### Reporting Bugs

If you find a bug, please open an [issue][issues] and provide the following information:

*   A clear and descriptive title.
*   A detailed description of the problem.
*   Steps to reproduce the bug.
*   The expected behavior.
*   The actual behavior.
*   Your environment (OS, Rust version, etc.).

### Suggesting Enhancements

If you have an idea for a new feature, please open an [issue][issues] and provide the following information:

*   A clear and descriptive title.
*   A detailed description of the proposed feature.
*   The problem that the feature would solve.
*   Any alternative solutions or features you've considered.

## Development

### Setting Up the Environment

To get started with development, you will need to have Rust and Cargo installed. You can find instructions on how to do that [here](https://www.rust-lang.org/tools/install).

Once you have Rust and Cargo installed, you can clone the repository and build the project:

```bash
git clone https://github.com/echozyr2001/CosmoFlow.git
cd CosmoFlow
cargo build
```

### Running Tests

You can run the tests with the following command:

```bash
cargo install cargo-nextest
cargo nextest run --all-features --workspace
```

or

```bash
cargo test
```

### Submitting Changes

1.  Fork the repository.
2.  Create a new branch (`git checkout -b my-new-feature`).
3.  Make your changes.
4.  Run the tests (`cargo test`).
5.  Commit your changes (`git commit -am 'Add some feature'`).
6.  Push to the branch (`git push origin my-new-feature`).
7.  Create a new Pull Request.

## Code of Conduct

This project and everyone participating in it is governed by the [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to [echo.zyr.2001@gmail.com](mailto:echo.zyr.2001@gmail.com).

[issues]: https://github.com/echozyr2001/CosmoFlow/issues