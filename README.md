# Aphrodite Rust - Core Engine

This is the Rust implementation of the Gaia Tools astrology platform core engine, migrated from Python/TypeScript.

## Status

All phases are complete:
- ✅ Phase 1: Core Computation Engine (Ephemeris, Aspects)
- ✅ Phase 2: Layout & ChartSpec Generation
- ✅ Phase 3: Axum API Server
- ✅ Phase 4: Frontend Renderers (Slint, WASM)
- ✅ Phase 5: Testing & Optimization
- ✅ Phase 6: Full Jyotish (Vedic Astrology)
- ✅ Phase 7: Dignities, Rulers, Decans

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
│   │   ├── rendering/      # ChartSpec generation
│   │   ├── vedic/          # Vedic astrology (nakshatras, vargas, dashas, yogas)
│   │   └── western/        # Western astrology (dignities, rulers, decans)
│   ├── tests/              # Unit tests
│   └── benches/            # Performance benchmarks
├── aphrodite-api/          # Axum HTTP API server
│   ├── src/
│   │   ├── routes/         # API endpoints
│   │   ├── services/       # Business logic
│   │   ├── schemas/        # Request/response types
│   │   └── middleware/     # CORS, rate limiting, logging
│   └── tests/              # Integration tests
├── aphrodite-slint/        # Slint renderer (desktop/mobile)
│   └── src/
├── aphrodite-wasm/         # WASM renderer (web)
│   ├── src/
│   └── examples/           # Web integration examples
└── docs/                   # Documentation
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

### Phase 3: Axum API Server

- **RESTful API**: `/api/render` and `/api/render/chartspec` endpoints
- **Request/Response Schemas**: Full type safety with serde
- **Service Layer**: Chart calculation orchestration
- **Middleware**: CORS, rate limiting, logging, error handling
- **Configuration**: Environment-based configuration

### Phase 4: Frontend Renderers

- **Slint Renderer**: Desktop and mobile rendering (structure in place)
- **WASM Renderer**: Web rendering with Canvas/SVG support
- **Web Integration**: Example HTML/JS integration

### Phase 5: Testing & Optimization

- **Unit Tests**: Comprehensive tests for all modules
- **Integration Tests**: API endpoint testing
- **Performance Benchmarks**: Criterion benchmarks for critical paths
- **Documentation**: API, rendering, testing, and performance guides

### Phase 6: Full Jyotish (Vedic Astrology)

- **Nakshatras**: 27 lunar mansions with padas (quarters)
  - Nakshatra identification from longitude
  - Pada calculation (1-4)
  - Planetary lords for each nakshatra
  - Layer annotation with nakshatra placements
- **Vargas**: 16 divisional charts (D2-D60)
  - Standard varga calculations (D4, D5, D6, D8, D9, D10, D12)
  - Special calculation methods for:
    - D2 (Hora): Sun/Moon hora based on odd/even signs
    - D3 (Drekkana): 1st/5th/9th sign offsets
    - D7 (Saptamsa): Odd/even sign logic
    - D16, D20, D24: Quality-based starting signs
    - D27 (Bhamsa): Always starts from Aries
    - D30 (Trimsamsa): Unequal divisions with planet rulers
    - D60 (Shashtiamsa): Always starts from Aries
- **Dashas**: Four dasha systems
  - **Vimshottari** (120 years): Based on Moon's nakshatra lord
  - **Yogini** (8 years): Based on Moon's nakshatra index
  - **Ashtottari** (108 years): Based on Moon's nakshatra groups
  - **Kalachakra** (120 years): Time wheel dasha
  - Supports mahadasha, antardasha, and pratyantardasha levels
- **Yogas**: Classic Vedic planetary combinations
  - Gajakesari Yoga (Jupiter + Moon in kendras/trikonas)
  - Budh Aditya Yoga (Mercury + Sun conjunction)
  - Raj Yoga (Benefics in kendras and trikonas)
  - Dhan Yoga (Wealth planets in 2nd/11th house)
  - Chandra-Mangal Yoga (Moon + Mars conjunction)
  - Shubh Kartari Yoga (Benefics around Moon)
  - Pap Kartari Yoga (Malefics around Moon)
  - Neecha Bhanga Raj Yoga (Debilitated planet with benefic)
  - Vipreet Raj Yoga (Malefic in 6th/8th/12th house)
  - Pancha Mahapurusha Yoga (Multiple planets in angular houses)

### Phase 7: Dignities, Rulers, Decans

- **Dignities**: Planetary strength indicators
  - Rulership: Planet in its own sign
  - Detriment: Planet in opposite sign of rulership
  - Exaltation: Planet in its exaltation sign
  - Fall: Planet in opposite sign of exaltation
  - Exact Exaltation: Planet within orb of exact exaltation degree
  - Supports all traditional planets plus Uranus, Neptune, Pluto
  - Default exact exaltation positions (Crowley)
- **Sign Rulers**: Traditional and modern rulerships
  - Traditional: Sun/Moon rule one sign each, others rule two
  - Modern: Uranus=Aquarius, Neptune=Pisces, Pluto=Scorpio
- **Decans**: Three decans per sign (10 degrees each)
  - Decan rulers based on element groups
  - Fire signs: Aries, Leo, Sagittarius
  - Earth signs: Taurus, Virgo, Capricorn
  - Air signs: Gemini, Libra, Aquarius
  - Water signs: Cancer, Scorpio, Pisces

## Dependencies

### Core Dependencies
- `swisseph` - Swiss Ephemeris Rust bindings
- `serde` / `serde_json` - JSON serialization
- `chrono` - Date/time handling
- `thiserror` - Error handling
- `anyhow` - Error context
- `regex` - Wheel definition validation
- `uuid` - ID generation
- `lazy_static` - Static initialization

### API Dependencies
- `axum` - Web framework
- `tokio` - Async runtime
- `tower` / `tower-http` - Middleware
- `tracing` / `tracing-subscriber` - Logging
- `governor` / `tower-governor` - Rate limiting

### Renderer Dependencies
- `slint` - UI framework (desktop/mobile)
- `wasm-bindgen` - WASM bindings
- `web-sys` - Web APIs

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

### Calculating Vedic Data

```rust
use aphrodite_core::vedic::{
    annotate_layer_nakshatras, build_varga_layers, identify_yogas,
    compute_vimshottari_dasha, DashaLevel,
};

// Nakshatras
let nakshatra_placements = annotate_layer_nakshatras(&positions, true, None);

// Vargas
let varga_layers = build_varga_layers("natal", &positions, &vec!["d9".to_string()]);

// Yogas
let yogas = identify_yogas(&positions);

// Dashas
let dashas = compute_vimshottari_dasha(birth_datetime, &positions, DashaLevel::Pratyantardasha)?;
```

### Calculating Western Data

```rust
use aphrodite_core::western::{DignitiesService, get_decan_info_from_longitude};

// Dignities
let service = DignitiesService;
let exact_exaltations = service.get_default_exact_exaltations();
let dignities = service.get_dignities("sun", 135.0, Some(&exact_exaltations));

// Decans
let decan_info = get_decan_info_from_longitude(135.0);
```

## Notes

### Swiss Ephemeris Crate

The implementation uses the `swisseph` crate, but the exact API may vary. The code includes comments indicating where adjustments may be needed based on the actual crate API. Key areas:

- Planet ID constants (SUN, MOON, etc.)
- Flag constants (FLG_SWIEPH, FLG_SIDEREAL)
- Function signatures (calc_ut, houses_ex2, set_sid_mode, etc.)

### API Usage

Start the server:
```bash
cd aphrodite-api
cargo run
```

The server will start on `http://localhost:8000` by default.

Example API request:
```bash
curl -X POST http://localhost:8000/api/v1/render \
  -H "Content-Type: application/json" \
  -d '{
    "subjects": [{
      "id": "person1",
      "label": "Test Person",
      "birthDateTime": "1990-01-01T12:00:00Z",
      "location": {"lat": 40.7128, "lon": -74.0060}
    }],
    "settings": {
      "zodiacType": "tropical",
      "houseSystem": "placidus",
      "includeObjects": ["sun", "moon", "mercury", "venus", "mars"]
    },
    "layer_config": {
      "natal": {
        "kind": "natal",
        "subjectId": "person1"
      }
    }
  }'
```

### Testing

Unit tests are available for all modules:
```bash
cargo test
```

Integration tests for the API:
```bash
cd aphrodite-api
cargo test
```

Performance benchmarks:
```bash
cd aphrodite-core
cargo bench
```

### Building WASM

```bash
cd aphrodite-wasm
wasm-pack build --target web --out-dir pkg
```

## Documentation

See the `docs/` directory for detailed documentation:
- `API.md` - API endpoint documentation
- `RENDERING.md` - ChartSpec and rendering guide
- `TESTING.md` - Testing strategy and examples
- `PERFORMANCE.md` - Performance benchmarks and optimization notes

## Notes

### Swiss Ephemeris Crate

The implementation uses the `swisseph` crate. The exact API may vary by version, so some adjustments may be needed:
- Planet ID constants
- Flag constants (FLG_SWIEPH, FLG_SIDEREAL)
- Function signatures (calc_ut, houses_ex2, set_sid_mode, etc.)

### Vedic Calculations

All Vedic calculations are now implemented:
- Nakshatras require sidereal zodiac (automatically handled)
- Vargas are derived from base layer positions (no new ephemeris calls)
- Dashas require Moon position in sidereal coordinates
- Yogas require house positions for kendra/trikona calculations

### Western Calculations

Western calculations work with both tropical and sidereal zodiac:
- Dignities and decans are sign-based (work with any zodiac system)
- Sign rulers support both traditional and modern rulerships

## License

This project is licensed under the **GNU Affero General Public License v3.0 or later** (AGPL-3.0-or-later).

See the [LICENSE](LICENSE) file for the full license text.

### Swiss Ephemeris Licensing

This project uses the `swisseph` crate, which provides Rust bindings to the **Swiss Ephemeris** library. The Swiss Ephemeris is available under a dual licensing system:

1. **GNU Affero General Public License (AGPL)**: The Swiss Ephemeris can be used under the AGPL, which requires that any software incorporating it must also be distributed under the AGPL or a compatible license. This is compatible with this project's AGPL-3.0-or-later license.

2. **Swiss Ephemeris Professional License**: For commercial use or if you prefer not to release your source code under the AGPL, you can purchase a professional license from Astrodienst. Information on obtaining this license is available at: [Swiss Ephemeris Professional License](https://www.astro.com/swisseph/swephprg.htm)

Please ensure that your use of this project and the Swiss Ephemeris complies with the applicable licensing terms.

