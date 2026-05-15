# GitHub Actions CI/CD Configuration

## Overview

This directory contains GitHub Actions workflows for automated testing, coverage reporting, and performance benchmarking of the aw-datastore testing framework.

## Workflows

### 1. test.yml - Main Test Suite ⭐

**Triggers**: Push to master/main/dev, Pull Requests  
**Run time**: ~10-15 minutes

**Jobs**:
- `test-sqlite`: Tests on Ubuntu, macOS, Windows (parallel)
- `test-postgres`: PostgreSQL integration tests on Ubuntu with Docker
- `lint`: Format checking and Clippy linting
- `test-all`: Summary job (ensures all pass)

**Features**:
- Cross-platform testing (3 OS simultaneously)
- Cargo caching for 60-70% speed improvement
- Parallel execution
- Automatic status reporting

### 2. coverage.yml - Code Coverage

**Triggers**: Push to master/main/dev, Pull Requests  
**Run time**: ~5 minutes

**Features**:
- Uses `cargo-tarpaulin` for coverage metrics
- Automatic upload to Codecov
- HTML report artifacts (30-day retention)
- PostgreSQL integration test coverage

### 3. benchmark.yml - Performance Benchmarks

**Triggers**: Push + Weekly schedule (Mondays 2 AM UTC)  
**Run time**: ~2 minutes

**Features**:
- SQLite benchmark runs
- PostgreSQL benchmark runs
- Performance metric comparison
- Weekly automated runs

## Setup Instructions

### 1. GitHub Repository Setup (Already Done ✅)

Actions are automatically enabled. No additional setup needed.

### 2. Optional: Codecov Integration

For automatic coverage uploading:

```bash
# Visit codecov.io and add your repository
https://codecov.io/github/new
```

For private repositories, you'll need a Codecov token (auto-detected for public repos).

### 3. Enable Workflow Artifacts

Artifacts are automatically configured with:
- Coverage reports: 30-day retention
- Benchmark results: 30-day retention

## Monitoring

### View Workflow Runs

**Actions Tab**: https://github.com/ActivityWatch/aw-server-rust/actions

**View specific workflow**:
```
https://github.com/ActivityWatch/aw-server-rust/actions/workflows/test.yml
```

### Status Badges (Optional)

Add to README.md:

```markdown
[![Tests](https://github.com/ActivityWatch/aw-server-rust/actions/workflows/test.yml/badge.svg)](https://github.com/ActivityWatch/aw-server-rust/actions/workflows/test.yml)
[![Coverage](https://codecov.io/gh/ActivityWatch/aw-server-rust/badge.svg)](https://codecov.io/gh/ActivityWatch/aw-server-rust)
```

## Troubleshooting

### Workflow fails on SQLite tests

**Cause**: Usually a code change broke tests  
**Solution**: 
```bash
cargo test --lib --tests --features sqlite
```

### PostgreSQL service fails to start

**Cause**: Port 5432 already in use  
**Solution**: Workflows auto-manage this, but locally:
```bash
make postgres-stop
make postgres-start
```

### Coverage upload fails

**Cause**: Codecov token issues or network error  
**Solution**: Not critical - tests still pass, coverage report is local

### Benchmark job times out

**Cause**: GitHub runner overloaded  
**Solution**: Benchmarks are optional; reduce frequency in `benchmark.yml`

## Performance Metrics

### Typical Run Times

| Workflow | Time | Frequency |
|----------|------|-----------|
| test.yml (all jobs) | 10-15 min | Every push/PR |
| coverage.yml | 5 min | Every push/PR |
| benchmark.yml | 2 min | Weekly + PR |

### Optimization Tips

- Cargo caching reduces build time by ~60%
- Parallel OS testing saves ~5 min
- Using `--lib --tests` avoids doc tests

## Continuous Integration Details

### Test Matrix

```yaml
SQLite Tests:
  - ubuntu-latest
  - macos-latest  
  - windows-latest

PostgreSQL Tests:
  - ubuntu-latest (with Docker PostgreSQL)

Lint:
  - ubuntu-latest
```

### Environment Variables

**For PostgreSQL**:
```
DATABASE_URL=postgresql://test:test@localhost:5432/aw_test
```

**GitHub Actions**:
```
RUST_BACKTRACE=1
CARGO_TERM_COLOR=always
```

## Integration with Development

### Before committing:
```bash
make lint        # Pass locally first
make test-sqlite # Quick check
```

### Before pushing:
```bash
make test-postgres  # Full integration test
```

### GitHub Actions will then:
1. Run tests on 3 platforms
2. Check code quality
3. Generate coverage report
4. Post results to PR

## Maintenance

### Update Rust version

Edit `.github/workflows/test.yml`:
```yaml
- uses: dtolnay/rust-toolchain@stable
```

To use specific version:
```yaml
- uses: dtolnay/rust-toolchain@1.70.0
```

### Update PostgreSQL version

Edit `.github/workflows/test.yml`:
```yaml
postgres:
  image: postgres:15-alpine  # Change version here
```

### Modify test frequency

Edit each workflow's `schedule` section:
```yaml
schedule:
  - cron: '0 2 * * 1'  # Mon 2AM UTC
```

## Performance Report Example

When benchmarks run, output includes:

```
=== SQLite Benchmarks ===
Benchmarking insert_event: Collected 100 samples
time: [123.45 ms 124.56 ms 125.67 ms]

=== PostgreSQL Benchmarks ===
Benchmarking insert_event: Collected 100 samples
time: [345.67 ms 346.78 ms 347.89 ms]
```

## See Also

- [Testing Guide](./TESTING.md)
- [Contributing Guide](./CONTRIBUTING.md)
- [Main README](./README.md)
