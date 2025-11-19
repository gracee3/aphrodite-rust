# Testing Guide

## Running Tests

### Unit Tests

```bash
cd aphrodite-core
cargo test
```

### Integration Tests

```bash
cd aphrodite-api
cargo test
```

### Benchmarks

```bash
cd aphrodite-core
cargo bench
```

## Test Structure

### Unit Tests

Located in `aphrodite-core/tests/`:
- `ephemeris_tests.rs` - Ephemeris calculation tests
- `aspect_tests.rs` - Aspect calculation tests
- `layout_tests.rs` - Wheel assembly tests
- `rendering_tests.rs` - ChartSpec generation tests

### Integration Tests

Located in `aphrodite-api/tests/integration/`:
- `render_tests.rs` - API endpoint tests

### Precision Tests

Located in `aphrodite-core/tests/precision/`:
- `ephemeris_precision.rs` - Compare Rust vs Python results

## Test Requirements

Some tests require:
- Swiss Ephemeris data files (marked with `#[ignore]`)
- Python environment for precision comparison
- Running API server for integration tests

## Running Specific Tests

```bash
# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test '*'

# Run ignored tests (requires setup)
cargo test -- --ignored
```

