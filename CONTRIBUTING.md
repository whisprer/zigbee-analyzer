\# Contributing to Secure Delete



Thanks for your interest in improving `woflOS`!



\## Code Style

\- Follow \*\*Rust 2021\*\* edition conventions.

\- Run `cargo fmt` before commits.

\- Ensure all warnings are resolved (`cargo clippy -- -D warnings`).

\- Keep logic minimal — avoid unnecessary dependencies.



\## Development Workflow

1\. Fork the repository.

2\. Create a feature branch:

&nbsp;  ```bash

&nbsp;  git checkout -b feature/your-feature

Commit changes with clear, conventional messages:



vbnet

Copy code

feat: add macOS-specific permission fix

fix: handle readonly directories on Windows

Push and open a Pull Request against main.



Testing

Run the build and ensure no regressions:



bash

Copy code

cargo build --release

cargo test

For manual verification:



Create temporary files.



Compare before/after with a hex viewer or recovery utility.



Documentation

If you add a feature, please update README.md accordingly.



Communication

Open an Issue for:



Feature requests



Bug reports



Questions or clarifications



Thanks for helping keep Secure-Delete clean, efficient, and reliable!

