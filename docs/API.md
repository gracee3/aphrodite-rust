# Aphrodite API Documentation

## Overview

The Aphrodite API provides endpoints for calculating astrological charts and generating chart specifications.

## Base URL

```
http://localhost:8000
```

## Endpoints

### Health Check

#### `GET /health`

Returns the health status of the API.

**Response:**
```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

### API Info

#### `GET /`

Returns API information.

**Response:**
```json
{
  "name": "Aphrodite API",
  "version": "0.1.0",
  "description": "Rust-based astrology charting API"
}
```

### Render Ephemeris

#### `POST /api/render`

Calculate ephemeris positions for a chart.

**Request Body:**
```json
{
  "subjects": [
    {
      "id": "subject1",
      "label": "Test Subject",
      "birthDateTime": "1990-01-01T12:00:00Z",
      "birthTimezone": "UTC",
      "location": {
        "name": "New York",
        "lat": 40.7128,
        "lon": -74.0060
      }
    }
  ],
  "settings": {
    "zodiacType": "tropical",
    "houseSystem": "placidus",
    "includeObjects": ["sun", "moon", "mercury", "venus", "mars"],
    "orbSettings": {
      "conjunction": 8.0,
      "opposition": 8.0,
      "trine": 7.0,
      "square": 6.0,
      "sextile": 4.0
    }
  },
  "layer_config": {
    "natal": {
      "kind": "natal",
      "subjectId": "subject1"
    }
  }
}
```

**Response:**
```json
{
  "layers": {
    "natal": {
      "id": "natal",
      "kind": "natal",
      "dateTime": "1990-01-01T12:00:00Z",
      "location": {
        "lat": 40.7128,
        "lon": -74.0060
      },
      "positions": {
        "planets": {
          "sun": {
            "lon": 280.5,
            "lat": 0.0,
            "speedLon": 1.0,
            "retrograde": false
          }
        },
        "houses": {
          "system": "placidus",
          "cusps": {
            "1": 120.0,
            "2": 150.0
          },
          "angles": {
            "asc": 120.0,
            "mc": 30.0
          }
        }
      }
    }
  },
  "settings": {
    "zodiacType": "tropical",
    "houseSystem": "placidus"
  }
}
```

### Render ChartSpec

#### `POST /api/render/chartspec`

Generate a complete ChartSpec for rendering.

**Request Body:** Same as `/api/render`

**Response:**
```json
{
  "spec": {
    "width": 800,
    "height": 800,
    "center": { "x": 400, "y": 400 },
    "rotation_offset": 0,
    "background_color": { "r": 0, "g": 0, "b": 0, "a": 255 },
    "shapes": [...],
    "metadata": {...}
  },
  "ephemeris": {...}
}
```

## Error Responses

All errors follow this format:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Error description",
    "correlation_id": "uuid"
  }
}
```

### Error Codes

- `VALIDATION_ERROR` - Request validation failed (400)
- `CALCULATION_ERROR` - Ephemeris calculation failed (400)
- `NOT_FOUND` - Resource not found (404)
- `RATE_LIMIT_EXCEEDED` - Rate limit exceeded (429)
- `INTERNAL_ERROR` - Server error (500)

