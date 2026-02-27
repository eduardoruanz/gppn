# Contributing to Veritas

Thank you for your interest in contributing to the Global Payment Protocol Network!

## Development Setup

1. **Install toolchain**: Rust 1.75+, Go 1.21+, protoc, Node.js 18+
2. **Clone the repository**: `git clone https://github.com/veritas-protocol/veritas.git`
3. **Build**: `cargo build --workspace`
4. **Test**: `cargo test --workspace`

## Code Standards

### Rust
- Run `cargo fmt` before committing
- Run `cargo clippy --workspace -- -D warnings` — zero warnings policy
- All public APIs must have doc comments
- Use `thiserror` for error types — no `unwrap()` in production code
- Target 90%+ test coverage for core crates

### Go
- Run `go fmt` and `go vet` before committing
- Follow standard Go project layout
- Write table-driven tests

### TypeScript
- Run `npm run lint` before committing
- Use strict TypeScript (`strict: true`)
- Write tests with vitest

## Pull Request Process

1. Fork the repository and create a feature branch
2. Write tests for new functionality
3. Ensure all CI checks pass
4. Submit a PR with a clear description of the changes

## Architecture Decisions

Major changes to the protocol or architecture should be proposed as a VIP (Veritas Improvement Proposal) in `spec/vips/`.

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.
