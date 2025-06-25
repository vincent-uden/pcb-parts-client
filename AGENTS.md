# Agent Guidelines for PCB Parts Client

## Build/Test Commands
- `cargo build` - Build all workspace crates
- `cargo run --bin gui` - Run GUI application  
- `cargo run --bin cli` - Run CLI application
- `cargo test` - Run all tests
- `cargo test -p common` - Run tests for specific crate
- `cargo test can_parse_altium_bom` - Run single test by name
- `cargo clippy` - Run linter
- `cargo fmt` - Format code

## Code Style Guidelines
- **Imports**: Group std, external crates, then local modules with blank lines between
- **Naming**: snake_case for functions/variables, PascalCase for types, SCREAMING_SNAKE_CASE for constants
- **Types**: Use explicit types for public APIs, derive Debug/Clone/Serialize/Deserialize for data structs
- **Error Handling**: Use `anyhow::Result<T>` for functions that can fail, `?` operator for propagation
- **Serde**: Use `#[serde(rename_all = "camelCase")]` for API compatibility
- **Documentation**: Add doc comments for public APIs
- **Testing**: Place tests in `#[cfg(test)]` modules, use descriptive test names

## Project Structure
- Workspace with 3 crates: `gui` (Iced app), `cli`, `common` (shared models/logic)
- Models in `common/src/models.rs` with Tabled derives for display
- Network operations in `common/src/network.rs`
- GUI uses Iced framework with Tokyo Night dark theme