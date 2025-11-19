# Rendering Guide

## Overview

Aphrodite provides platform-agnostic chart rendering through the ChartSpec format. Each frontend platform implements a renderer that consumes ChartSpec and draws it using platform-appropriate tools.

## ChartSpec Format

ChartSpec is a declarative JSON structure describing all chart elements:

```json
{
  "width": 800,
  "height": 800,
  "center": { "x": 400, "y": 400 },
  "rotation_offset": 0,
  "background_color": { "r": 0, "g": 0, "b": 0, "a": 255 },
  "shapes": [
    {
      "type": "Circle",
      "center": { "x": 100, "y": 200 },
      "radius": 50,
      "fill": { "r": 255, "g": 0, "b": 0, "a": 255 }
    }
  ]
}
```

## Shape Types

### Circle
```json
{
  "type": "Circle",
  "center": { "x": 100, "y": 200 },
  "radius": 50,
  "fill": { "r": 255, "g": 0, "b": 0, "a": 255 },
  "stroke": {
    "color": { "r": 255, "g": 255, "b": 255, "a": 255 },
    "width": 2
  }
}
```

### Arc
```json
{
  "type": "Arc",
  "center": { "x": 400, "y": 400 },
  "radius_inner": 200,
  "radius_outer": 250,
  "start_angle": 0,
  "end_angle": 30,
  "fill": { "r": 255, "g": 0, "b": 0, "a": 255 }
}
```

### Line
```json
{
  "type": "Line",
  "from": { "x": 100, "y": 100 },
  "to": { "x": 200, "y": 200 },
  "stroke": {
    "color": { "r": 255, "g": 255, "b": 255, "a": 255 },
    "width": 2
  }
}
```

### Text
```json
{
  "type": "Text",
  "position": { "x": 100, "y": 100 },
  "content": "Planet Name",
  "size": 12,
  "color": { "r": 255, "g": 255, "b": 255, "a": 255 },
  "anchor": "Middle",
  "rotation": 45
}
```

### PlanetGlyph
```json
{
  "type": "PlanetGlyph",
  "center": { "x": 100, "y": 200 },
  "planet_id": "sun",
  "size": 12,
  "color": { "r": 255, "g": 215, "b": 0, "a": 255 },
  "retrograde": false
}
```

## WASM Renderer

### Usage

```javascript
import init, { ChartRenderer } from './pkg/aphrodite_wasm.js';

await init();

const canvas = document.getElementById('chart-canvas');
const ctx = canvas.getContext('2d');

// Fetch ChartSpec from API
const response = await fetch('/api/v1/render/chartspec', {...});
const data = await response.json();

// Create renderer and render
const renderer = new ChartRenderer(JSON.stringify(data.spec));
renderer.render_to_canvas(ctx);
```

## Slint Renderer

### Usage

```rust
use aphrodite_slint::SlintChartRenderer;
use aphrodite_core::rendering::ChartSpec;

let renderer = SlintChartRenderer::new(chart_spec);
// Render to Slint component
```

## Coordinate System

- Origin (0, 0) is at top-left
- X increases to the right
- Y increases downward
- Angles are in degrees, 0° = top (12 o'clock), clockwise
- Astronomical angles are converted to SVG/cartesian coordinates

