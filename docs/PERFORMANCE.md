# Performance Notes

## Benchmarks

Performance benchmarks are available in `aphrodite-core/benches/`.

Run benchmarks with:
```bash
cargo bench
```

## Expected Performance

### Ephemeris Calculations

- Single planet calculation: < 1ms
- Full chart (10 planets + houses): < 10ms
- Significantly faster than Python `pyswisseph` due to Rust's performance

### Aspect Calculations

- Single aspect: < 0.1ms
- Full chart (all aspects): < 5ms
- Faster than TypeScript due to compiled code

### API Endpoints

- `/api/render`: < 50ms (including ephemeris + aspects)
- `/api/render/chartspec`: < 100ms (including wheel assembly + ChartSpec generation)

## Optimization Opportunities

1. **Caching**: Ephemeris calculations can be cached for repeated requests
2. **Parallelization**: Aspect calculations can be parallelized with `rayon`
3. **WASM Size**: Use `wasm-opt` to reduce binary size
4. **Response Compression**: Enable gzip compression for API responses

## Memory Usage

- Typical chart calculation: < 10MB
- WASM binary size: < 2MB (optimized)
- API server memory: < 50MB (idle)

