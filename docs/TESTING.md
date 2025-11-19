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
cargo test --test integration
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
- `render_tests.rs` - Comprehensive API endpoint tests (60 tests total)

The integration test suite includes:

#### Health and Info Endpoints (4 tests)
- Health check endpoint structure validation
- API info endpoint with full response validation
- CORS headers testing
- HTTP method validation

#### Render Endpoint - Success Cases (8 tests, 7 ignored)
- Basic render functionality
- Response structure validation
- All planets included
- Different house systems (placidus, whole_sign, koch, equal, etc.)
- Tropical vs sidereal zodiac
- Multiple subjects
- Transit and progressed layers
- ChartSpec endpoint with structure validation

#### Validation Error Cases (24 tests)
- Missing/empty subjects
- Duplicate subject IDs
- Invalid house systems
- Invalid zodiac types
- Invalid ayanamsas
- Invalid coordinates (latitude/longitude boundaries, NaN)
- Invalid datetime formats
- Date out of range
- Invalid orb settings (too high, negative, NaN)
- Missing/invalid layer configs
- Invalid layer kinds
- Natal layer validation (missing subjectId, invalid subjectId)
- Transit/progressed layer validation (missing datetime)
- Invalid planet names

#### Edge Cases and Advanced Scenarios (13 tests)
- Missing request body
- Invalid JSON
- Missing required fields
- Boundary coordinate values
- Boundary orb settings
- Cache functionality (ignored - requires Swiss Ephemeris)
- Sequential requests (server stability)
- Settings merge
- Empty includeObjects
- Default orb settings
- Optional location
- Timezone handling
- Large requests

#### Error Response Structure (2 tests)
- Error response format validation
- Different error types

#### HTTP Method Tests (2 tests)
- Wrong HTTP methods
- Content type handling

#### Full Workflow Integration Tests (2 tests, ignored)
- Complete natal chart workflow
- Composite chart workflow

### Precision Tests

Located in `aphrodite-core/tests/precision/`:
- `ephemeris_precision.rs` - Compare Rust vs Python results

## Test Requirements

Some tests require:
- **Swiss Ephemeris data files** (15 tests marked with `#[ignore]`)
  - These tests validate actual ephemeris calculations and require Swiss Ephemeris files to be installed
  - To run: `cargo test --test integration -- --ignored`
- Python environment for precision comparison
- **axum-test 16.0+** (compatible with axum 0.7)

## Running Specific Tests

```bash
# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration

# Run all tests including ignored ones (requires Swiss Ephemeris)
cargo test --test integration -- --ignored

# Run a specific test
cargo test --test integration test_health_endpoint

# Run tests with output
cargo test --test integration -- --nocapture
```

## Test Coverage

The integration test suite provides comprehensive coverage:

- **45 passing tests** - Validation, error handling, edge cases
- **15 ignored tests** - Require Swiss Ephemeris files for full functionality
- **Total: 60 tests**

### Test Categories

1. **Validation Tests**: Ensure all input validation works correctly
2. **Success Path Tests**: Verify correct responses for valid requests
3. **Error Handling Tests**: Confirm proper error responses and structure
4. **Edge Case Tests**: Boundary conditions, optional fields, defaults
5. **Integration Tests**: Full workflow from request to response

## Test Framework

The integration tests use:
- **axum-test 16.0** - For testing Axum applications (compatible with axum 0.7)
- **tokio** - Async runtime for async test functions
- **serde_json** - JSON request/response handling

## Writing New Tests

When adding new endpoints or features:

1. Add tests to `aphrodite-api/tests/integration/render_tests.rs`
2. Follow the existing test structure and naming conventions
3. Mark tests requiring Swiss Ephemeris with `#[ignore]`
4. Test both success and error cases
5. Validate response structure, not just status codes
6. Handle both 400 (validation) and 500 (internal) errors appropriately

