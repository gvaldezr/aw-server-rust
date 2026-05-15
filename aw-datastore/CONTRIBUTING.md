# Contributing to aw-datastore Testing Framework

Thank you for your interest in contributing! This guide helps you get started.

## Development Setup

### Prerequisites
- Rust 1.56+ (MSRV: check `Cargo.toml`)
- PostgreSQL 13+ (for integration tests)
- Make (for convenient commands)
- Docker Compose (optional, for PostgreSQL in containers)

### Quick Start
```bash
# Clone the repository
git clone https://github.com/ActivityWatch/aw-datastore.git
cd aw-datastore

# Setup
make help                    # View available Make targets
cargo build                  # Build the crate

# Run tests
make test-sqlite            # Fast SQLite tests
make test-postgres          # PostgreSQL integration tests (requires running PostgreSQL)
make test                   # Run all tests
```

## Branch Workflow

1. **Create a feature branch**
   ```bash
   git checkout -b feat/your-feature-name
   ```

2. **Make your changes**
   - Follow Rust conventions
   - Add tests for new functionality
   - Update documentation

3. **Test locally**
   ```bash
   make lint                # Run clippy and fmt check
   make test-sqlite         # Fast local tests
   make test-postgres       # Full integration tests
   ```

4. **Commit with conventional messages**
   ```bash
   git commit -m "feat: add new feature

   - Detailed explanation
   - Additional context"
   ```

5. **Push and create PR**
   ```bash
   git push -u origin feat/your-feature-name
   ```

## Test Standards

### Writing Tests

Tests should follow the parametrized pattern:

```rust
#[test]
fn test_my_feature_on_sqlite() {
    run_test_on_backend(TestBackend::Sqlite, test_my_feature);
}

#[test]
fn test_my_feature_on_postgres() {
    run_test_on_backend(TestBackend::Postgres, test_my_feature);
}

fn test_my_feature(fixture: &mut BackendFixture) {
    // Your test code here
    let bucket = fixture.create_test_bucket();
    // assertions...
}
```

### Test Naming Convention
- `test_<feature>_<scenario>` for core tests
- `test_<feature>_<scenario>_on_sqlite()` for SQLite wrapper
- `test_<feature>_<scenario>_on_postgres()` for PostgreSQL wrapper

### Running Specific Tests
```bash
cargo test test_my_feature --lib -- --nocapture
cargo test test_my_feature --test '*' -- --nocapture
```

## Code Style

### Formatting & Linting
```bash
cargo fmt                  # Format code
cargo clippy -- -D warnings  # Check for warnings (must be zero)
cargo test --doc           # Test documentation examples
```

### Style Guidelines
- Use meaningful variable names
- Comment complex logic
- Keep functions focused and small
- Handle errors explicitly (minimize `.unwrap()`)

## Documentation

- Update relevant `.md` files in `docs/`
- Add doc comments to public items
- Include examples in doc comments
- Keep README.md current

## Pull Request Process

1. **Ensure tests pass**
   ```bash
   make test-sqlite
   make test-postgres
   make lint
   ```

2. **Update documentation** if needed

3. **Create PR with description**
   - Clear title and description
   - Link related issues (#123)
   - Mention any breaking changes

4. **Address review feedback**
   - GitHub Actions will run automatically
   - Respond to reviewer comments
   - Push updates to your branch

## Debugging Tips

### Run tests with output
```bash
cargo test test_name -- --nocapture --test-threads=1
```

### Enable Rust backtrace
```bash
RUST_BACKTRACE=1 cargo test
```

### Test database issues
```bash
# Start PostgreSQL manually
make postgres-start
psql postgresql://test:test@localhost:5432/aw_test

# Stop when done
make postgres-stop
```

### Check test coverage
```bash
cargo tarpaulin --out Html
open tarpaulin-report.html
```

## Reporting Issues

1. **Check existing issues** first to avoid duplicates
2. **Use bug_report.md template** (automatically loaded on GitHub)
3. **Include**:
   - Clear description
   - Reproduction steps
   - Environment (OS, Rust version)
   - Error logs

## License

By contributing, you agree that your contributions are licensed under the MIT License.

## Questions?

- Check the [main README](README.md)
- See [testing guide](TESTING.md)
- Review [architecture docs](tests/README.md)
- Ask in issues or discussions

---

Thank you for contributing! 🎉
