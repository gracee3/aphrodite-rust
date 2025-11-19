# Aphrodite Rust - Core Engine

This is the Rust implementation of the Gaia Tools astrology platform core engine, migrated from Python/TypeScript.

## Status

Phases 1 & 2 are complete:
- ✅ Phase 1: Core Computation Engine (Ephemeris, Aspects)
- ✅ Phase 2: Layout & ChartSpec Generation

## Structure

```
aphrodite-rust/
├── Cargo.toml              # Workspace configuration
├── aphrodite-core/         # Main computation crate
│   ├── src/
│   │   ├── lib.rs
│   │   ├── ephemeris/      # Swiss Ephemeris adapter
│   │   ├── aspects/        # Aspect calculation engine
│   │   ├── layout/         # Wheel assembly & JSON loading
│   │   └── rendering/      # ChartSpec generation
│   └── Cargo.toml
└── aphrodite-api/          # Placeholder for Phase 3 (Axum server)
    └── Cargo.toml
```

## Features

### Phase 1: Core Computation Engine

- **Swiss Ephemeris Integration**: Uses `swisseph` crate for planetary and house calculations
- **Ephemeris Calculations**: 
  - Planetary positions (all major planets, Chiron, nodes)
  - House systems (Placidus, Whole Sign, Koch, Equal, Regiomontanus, Campanus, Alcabitius, Morinus)
  - Tropical/Sidereal zodiac support
  - Multiple ayanamsas (Lahiri, Fagan-Bradley, Raman, etc.)
- **Aspect Calculation Engine**:
  - Intra-layer and inter-layer aspects
  - Support for conjunction, opposition, trine, square, sextile
  - Orb settings per aspect type
  - Applying/separating detection
  - Retrograde detection

### Phase 2: Layout & ChartSpec Generation

- **Wheel Definition JSON Loading**: Load wheel definitions from JSON at runtime
- **Wheel Assembly**: Assemble wheels from definitions with resolved ring items
- **ChartSpec Generation**: Convert assembled wheels to declarative ChartSpec format
- **Visual Configuration**: Default colors, glyphs, and styling

## Dependencies

- `swisseph` - Swiss Ephemeris Rust bindings
- `serde` / `serde_json` - JSON serialization
- `chrono` - Date/time handling
- `thiserror` - Error handling
- `anyhow` - Error context
- `regex` - Wheel definition validation
- `uuid` - ID generation

## Usage

### Loading a Wheel Definition from JSON

```rust
use aphrodite_core::layout::load_wheel_definition_from_json;

let json = r#"
{
  "name": "Standard Natal Wheel",
  "rings": [
    {
      "slug": "ring_signs",
      "type": "signs",
      "label": "Zodiac Signs",
      "orderIndex": 0,
      "radiusInner": 0.85,
      "radiusOuter": 1.0,
      "dataSource": { "kind": "static_zodiac" }
    }
  ]
}
"#;

let wheel_def = load_wheel_definition_from_json(json)?;
```

### Calculating Ephemeris Positions

```rust
use aphrodite_core::ephemeris::{SwissEphemerisAdapter, EphemerisSettings, GeoLocation};
use chrono::Utc;

let mut adapter = SwissEphemerisAdapter::new(None)?;
let settings = EphemerisSettings {
    zodiac_type: "tropical".to_string(),
    ayanamsa: None,
    house_system: "placidus".to_string(),
    include_objects: vec!["sun".to_string(), "moon".to_string()],
};
let location = Some(GeoLocation { lat: 40.7128, lon: -74.0060 });
let positions = adapter.calc_positions(Utc::now(), location, &settings)?;
```

### Calculating Aspects

```rust
use aphrodite_core::aspects::{AspectCalculator, AspectSettings};
use std::collections::HashMap;

let calculator = AspectCalculator::new();
let orb_settings = HashMap::from([
    ("conjunction".to_string(), 8.0),
    ("opposition".to_string(), 8.0),
    ("trine".to_string(), 7.0),
    ("square".to_string(), 6.0),
    ("sextile".to_string(), 4.0),
]);
let settings = AspectSettings {
    orb_settings,
    include_objects: vec![],
    only_major: None,
};
let aspect_set = calculator.compute_intra_layer_aspects("natal", &positions, &settings);
```

### Generating ChartSpec

```rust
use aphrodite_core::rendering::ChartSpecGenerator;
use aphrodite_core::layout::WheelAssembler;

let wheel = WheelAssembler::build_wheel(&wheel_def, &positions_by_layer, &aspect_sets, None);
let generator = ChartSpecGenerator::new();
let spec = generator.generate(&wheel, &aspect_sets, 800.0, 800.0);
```

## Notes

### Swiss Ephemeris Crate

The implementation uses the `swisseph` crate, but the exact API may vary. The code includes comments indicating where adjustments may be needed based on the actual crate API. Key areas:

- Planet ID constants (SUN, MOON, etc.)
- Flag constants (FLG_SWIEPH, FLG_SIDEREAL)
- Function signatures (calc_ut, houses_ex2, set_sid_mode, etc.)

### Vedic Calculations

Vedic calculations (nakshatras, vargas, dashas, yogas) are deferred to Phase 6 as requested.

### Testing

Unit tests should be added for each module. Integration tests should compare Rust output with Python/TypeScript output to ensure precision matches.

## Next Steps (Phases 3-5)

- Phase 3: Axum API Server
- Phase 4: Frontend Renderers (Slint, WASM)
- Phase 5: Testing & Optimization
- Phase 6: Vedic Calculations

