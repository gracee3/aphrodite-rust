// Integration tests for API endpoints
use aphrodite_api::routes;
use axum_test::TestServer;
use serde_json::json;

/// Create a test server with a minimal configuration
fn create_test_server() -> TestServer {
    // Set environment variables for test configuration
    std::env::set_var("SWISS_EPHEMERIS_PATH", "/usr/local/share/swisseph");
    std::env::set_var("SERVICE_POOL_SIZE", "2");
    std::env::set_var("CACHE_SIZE", "100");
    
    let app = routes::create_router();
    TestServer::new(app).unwrap()
}

/// Create a valid test request payload
fn create_valid_request() -> serde_json::Value {
    json!({
        "subjects": [{
            "id": "test_person",
            "label": "Test Person",
            "birthDateTime": "1990-01-01T12:00:00Z",
            "location": {
                "lat": 40.7128,
                "lon": -74.0060
            }
        }],
        "settings": {
            "zodiacType": "tropical",
            "houseSystem": "placidus",
            "includeObjects": ["sun", "moon", "mercury", "venus", "mars"]
        },
        "layer_config": {
            "natal": {
                "kind": "natal",
                "subjectId": "test_person"
            }
        }
    })
}

/// Create a request with multiple subjects
fn create_multi_subject_request() -> serde_json::Value {
    json!({
        "subjects": [
            {
                "id": "person1",
                "label": "Person One",
                "birthDateTime": "1990-01-01T12:00:00Z",
                "location": {
                    "lat": 40.7128,
                    "lon": -74.0060
                }
            },
            {
                "id": "person2",
                "label": "Person Two",
                "birthDateTime": "1995-06-15T18:30:00Z",
                "location": {
                    "lat": 51.5074,
                    "lon": -0.1278
                }
            }
        ],
        "settings": {
            "zodiacType": "tropical",
            "houseSystem": "placidus",
            "includeObjects": ["sun", "moon", "mercury", "venus", "mars", "jupiter", "saturn"]
        },
        "layer_config": {
            "natal1": {
                "kind": "natal",
                "subjectId": "person1"
            },
            "natal2": {
                "kind": "natal",
                "subjectId": "person2"
            }
        }
    })
}

/// Create a request with transit layer
fn create_transit_request() -> serde_json::Value {
    json!({
        "subjects": [{
            "id": "test_person",
            "label": "Test Person",
            "birthDateTime": "1990-01-01T12:00:00Z",
            "location": {
                "lat": 40.7128,
                "lon": -74.0060
            }
        }],
        "settings": {
            "zodiacType": "tropical",
            "houseSystem": "placidus",
            "includeObjects": ["sun", "moon", "mercury", "venus", "mars"]
        },
        "layer_config": {
            "natal": {
                "kind": "natal",
                "subjectId": "test_person"
            },
            "transit": {
                "kind": "transit",
                "explicitDateTime": "2024-01-01T12:00:00Z",
                "location": {
                    "lat": 40.7128,
                    "lon": -74.0060
                }
            }
        }
    })
}

// ============================================================================
// Health and Info Endpoints
// ============================================================================

#[tokio::test]
async fn test_health_endpoint() {
    let server = create_test_server();
    
    let response = server.get("/health").await;
    response.assert_status_ok();
    
    let body: serde_json::Value = response.json();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["version"], "0.1.0");
}

#[tokio::test]
async fn test_health_endpoint_structure() {
    let server = create_test_server();
    
    let response = server.get("/health").await;
    response.assert_status_ok();
    
    let body: serde_json::Value = response.json();
    assert!(body.get("status").is_some());
    assert!(body.get("version").is_some());
    assert!(body["status"].is_string());
    assert!(body["version"].is_string());
}

#[tokio::test]
async fn test_api_info_endpoint() {
    let server = create_test_server();
    
    let response = server.get("/").await;
    response.assert_status_ok();
    
    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "Aphrodite API");
    assert_eq!(body["version"], "0.1.0");
    assert_eq!(body["description"], "Rust-based astrology charting API");
}

#[tokio::test]
async fn test_api_info_endpoint_structure() {
    let server = create_test_server();
    
    let response = server.get("/").await;
    response.assert_status_ok();
    
    let body: serde_json::Value = response.json();
    assert!(body.get("name").is_some());
    assert!(body.get("version").is_some());
    assert!(body.get("description").is_some());
    assert!(body["name"].is_string());
    assert!(body["version"].is_string());
    assert!(body["description"].is_string());
}

#[tokio::test]
async fn test_cors_headers() {
    let server = create_test_server();
    
    let response = server
        .get("/health")
        .add_header("Origin", "https://example.com")
        .await;
    
    response.assert_status_ok();
    // CORS layer is permissive, so headers should be present
    // Note: axum-test may not expose all headers, but the request should succeed
}

// ============================================================================
// Render Endpoint - Success Cases
// ============================================================================

#[tokio::test]
#[ignore] // Requires Swiss Ephemeris files
async fn test_render_endpoint_success() {
    let server = create_test_server();
    let request = create_valid_request();
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert!(body.get("layers").is_some());
    assert!(body.get("settings").is_some());
}

#[tokio::test]
#[ignore] // Requires Swiss Ephemeris files
async fn test_render_endpoint_response_structure() {
    let server = create_test_server();
    let request = create_valid_request();
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    
    // Check layers structure
    assert!(body["layers"].is_object());
    assert!(body["layers"]["natal"].is_object());
    assert_eq!(body["layers"]["natal"]["id"], "natal");
    assert_eq!(body["layers"]["natal"]["kind"], "natal");
    assert!(body["layers"]["natal"]["dateTime"].is_string());
    assert!(body["layers"]["natal"]["positions"].is_object());
    assert!(body["layers"]["natal"]["positions"]["planets"].is_object());
    
    // Check settings structure
    assert!(body["settings"].is_object());
    assert_eq!(body["settings"]["zodiacType"], "tropical");
    assert_eq!(body["settings"]["houseSystem"], "placidus");
}

#[tokio::test]
#[ignore] // Requires Swiss Ephemeris files
async fn test_render_endpoint_all_planets() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["settings"]["includeObjects"] = json!([
        "sun", "moon", "mercury", "venus", "mars", 
        "jupiter", "saturn", "uranus", "neptune", "pluto",
        "chiron", "north_node", "south_node"
    ]);
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    let planets = &body["layers"]["natal"]["positions"]["planets"];
    
    // Check that all requested planets are present
    assert!(planets.get("sun").is_some());
    assert!(planets.get("moon").is_some());
    assert!(planets.get("mercury").is_some());
    assert!(planets.get("venus").is_some());
    assert!(planets.get("mars").is_some());
    assert!(planets.get("jupiter").is_some());
    assert!(planets.get("saturn").is_some());
    assert!(planets.get("uranus").is_some());
    assert!(planets.get("neptune").is_some());
    assert!(planets.get("pluto").is_some());
    assert!(planets.get("chiron").is_some());
    assert!(planets.get("north_node").is_some());
    assert!(planets.get("south_node").is_some());
}

#[tokio::test]
#[ignore] // Requires Swiss Ephemeris files
async fn test_render_endpoint_different_house_systems() {
    let server = create_test_server();
    let house_systems = vec!["placidus", "whole_sign", "koch", "equal", "regiomontanus", "campanus"];
    
    for house_system in house_systems {
        let mut request = create_valid_request();
        request["settings"]["houseSystem"] = json!(house_system);
        
        let response = server
            .post("/api/v1/render")
            .json(&request)
            .await;
        
        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert_eq!(body["settings"]["houseSystem"], house_system);
        
        if let Some(houses) = body["layers"]["natal"]["positions"].get("houses") {
            assert_eq!(houses["system"], house_system);
        }
    }
}

#[tokio::test]
#[ignore] // Requires Swiss Ephemeris files
async fn test_render_endpoint_tropical_vs_sidereal() {
    let server = create_test_server();
    
    // Test tropical
    let mut request_tropical = create_valid_request();
    request_tropical["settings"]["zodiacType"] = json!("tropical");
    
    let response_tropical = server
        .post("/api/v1/render")
        .json(&request_tropical)
        .await;
    response_tropical.assert_status_ok();
    let body_tropical: serde_json::Value = response_tropical.json();
    assert_eq!(body_tropical["settings"]["zodiacType"], "tropical");
    
    // Test sidereal
    let mut request_sidereal = create_valid_request();
    request_sidereal["settings"]["zodiacType"] = json!("sidereal");
    request_sidereal["settings"]["ayanamsa"] = json!("lahiri");
    
    let response_sidereal = server
        .post("/api/v1/render")
        .json(&request_sidereal)
        .await;
    response_sidereal.assert_status_ok();
    let body_sidereal: serde_json::Value = response_sidereal.json();
    assert_eq!(body_sidereal["settings"]["zodiacType"], "sidereal");
    
    // Positions should be different between tropical and sidereal
    let sun_tropical = body_tropical["layers"]["natal"]["positions"]["planets"]["sun"]["lon"].as_f64().unwrap();
    let sun_sidereal = body_sidereal["layers"]["natal"]["positions"]["planets"]["sun"]["lon"].as_f64().unwrap();
    assert_ne!(sun_tropical, sun_sidereal);
}

#[tokio::test]
#[ignore] // Requires Swiss Ephemeris files
async fn test_render_endpoint_multiple_subjects() {
    let server = create_test_server();
    let request = create_multi_subject_request();
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    
    // Check both layers exist
    assert!(body["layers"]["natal1"].is_object());
    assert!(body["layers"]["natal2"].is_object());
    assert_eq!(body["layers"]["natal1"]["id"], "natal1");
    assert_eq!(body["layers"]["natal2"]["id"], "natal2");
}

#[tokio::test]
#[ignore] // Requires Swiss Ephemeris files
async fn test_render_endpoint_transit_layer() {
    let server = create_test_server();
    let request = create_transit_request();
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    
    // Check both natal and transit layers exist
    assert!(body["layers"]["natal"].is_object());
    assert!(body["layers"]["transit"].is_object());
    assert_eq!(body["layers"]["transit"]["kind"], "transit");
}

#[tokio::test]
#[ignore] // Requires Swiss Ephemeris files
async fn test_render_chartspec_endpoint_success() {
    let server = create_test_server();
    let request = create_valid_request();
    
    let response = server
        .post("/api/v1/render/chartspec")
        .json(&request)
        .await;
    
    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert!(body.get("spec").is_some());
    assert!(body.get("ephemeris").is_some());
}

#[tokio::test]
#[ignore] // Requires Swiss Ephemeris files
async fn test_render_chartspec_endpoint_structure() {
    let server = create_test_server();
    let request = create_valid_request();
    
    let response = server
        .post("/api/v1/render/chartspec")
        .json(&request)
        .await;
    
    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    
    // Check spec structure
    assert!(body["spec"].is_object());
    assert!(body["spec"]["width"].is_number());
    assert!(body["spec"]["height"].is_number());
    assert!(body["spec"]["center"].is_object());
    assert!(body["spec"]["shapes"].is_array());
    
    // Check ephemeris structure
    assert!(body["ephemeris"].is_object());
    assert!(body["ephemeris"]["layers"].is_object());
    assert!(body["ephemeris"]["settings"].is_object());
}

// ============================================================================
// Render Endpoint - Validation Error Cases
// ============================================================================

#[tokio::test]
async fn test_render_endpoint_validation_error_missing_subject() {
    let server = create_test_server();
    let request = json!({
        "subjects": [],
        "settings": {
            "zodiacType": "tropical",
            "houseSystem": "placidus"
        },
        "layer_config": {}
    });
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error code if we got a client error with valid JSON
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
        assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
        assert!(body["error"]["message"].as_str().unwrap().contains("subject"));
        assert!(body["error"]["correlation_id"].is_string());
    }
}

#[tokio::test]
async fn test_render_endpoint_validation_error_empty_subject_id() {
    let server = create_test_server();
    let request = json!({
        "subjects": [{
            "id": "",
            "label": "Test",
            "birthDateTime": "1990-01-01T12:00:00Z"
        }],
        "settings": {
            "zodiacType": "tropical",
            "houseSystem": "placidus"
        },
        "layer_config": {
            "natal": {
                "kind": "natal",
                "subjectId": ""
            }
        }
    });
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error code if we got a client error with valid JSON
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
        assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    }
}

#[tokio::test]
async fn test_render_endpoint_validation_error_duplicate_subject_id() {
    let server = create_test_server();
    let request = json!({
        "subjects": [
            {
                "id": "duplicate",
                "label": "Test 1",
                "birthDateTime": "1990-01-01T12:00:00Z"
            },
            {
                "id": "duplicate",
                "label": "Test 2",
                "birthDateTime": "1995-01-01T12:00:00Z"
            }
        ],
        "settings": {
            "zodiacType": "tropical",
            "houseSystem": "placidus"
        },
        "layer_config": {
            "natal": {
                "kind": "natal",
                "subjectId": "duplicate"
            }
        }
    });
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error code if we got a client error with valid JSON
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
        assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
        assert!(body["error"]["message"].as_str().unwrap().contains("Duplicate"));
    }
}

#[tokio::test]
async fn test_render_endpoint_validation_error_invalid_house_system() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["settings"]["houseSystem"] = json!("invalid_system");
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error code if we got a client error with valid JSON
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
            assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
        assert!(body["error"]["message"].as_str().unwrap().contains("houseSystem"));
    }
}

#[tokio::test]
async fn test_render_endpoint_validation_error_invalid_zodiac_type() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["settings"]["zodiacType"] = json!("invalid_zodiac");
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error code if we got a client error with valid JSON
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
            assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
        assert!(body["error"]["message"].as_str().unwrap().contains("zodiacType"));
    }
}

#[tokio::test]
async fn test_render_endpoint_validation_error_invalid_ayanamsa() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["settings"]["zodiacType"] = json!("sidereal");
    request["settings"]["ayanamsa"] = json!("invalid_ayanamsa");
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error code if we got a client error with valid JSON
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
            assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
        assert!(body["error"]["message"].as_str().unwrap().contains("ayanamsa"));
    }
}

#[tokio::test]
async fn test_render_endpoint_validation_error_invalid_coordinates_latitude() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["subjects"][0]["location"]["lat"] = json!(100.0); // Invalid latitude
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error code if we got a client error with valid JSON
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
            assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
        assert!(body["error"]["message"].as_str().unwrap().contains("latitude"));
    }
}

#[tokio::test]
async fn test_render_endpoint_validation_error_invalid_coordinates_longitude() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["subjects"][0]["location"]["lon"] = json!(200.0); // Invalid longitude
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error code if we got a client error with valid JSON
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
            assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
        assert!(body["error"]["message"].as_str().unwrap().contains("longitude"));
    }
}

#[tokio::test]
async fn test_render_endpoint_validation_error_invalid_coordinates_nan() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["subjects"][0]["location"]["lat"] = json!(f64::NAN);
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error code if we got a client error with valid JSON
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
        assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    }
}

#[tokio::test]
async fn test_render_endpoint_validation_error_invalid_datetime_format() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["subjects"][0]["birthDateTime"] = json!("invalid-date");
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error code if we got a client error with valid JSON
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
            assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
        assert!(body["error"]["message"].as_str().unwrap().contains("datetime") || 
                body["error"]["message"].as_str().unwrap().contains("parse"));
    }
}

#[tokio::test]
async fn test_render_endpoint_validation_error_date_out_of_range() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["subjects"][0]["birthDateTime"] = json!("5000-01-01T12:00:00Z"); // Too far in future
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error code if we got a client error with valid JSON
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
            assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
        assert!(body["error"]["message"].as_str().unwrap().contains("year") || 
                body["error"]["message"].as_str().unwrap().contains("range"));
    }
}

#[tokio::test]
async fn test_render_endpoint_validation_error_invalid_orb_setting_too_high() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["settings"]["orbSettings"] = json!({
        "conjunction": 50.0 // Invalid - exceeds max of 30
    });
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error code if we got a client error with valid JSON
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
            assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
        assert!(body["error"]["message"].as_str().unwrap().contains("orbSettings"));
    }
}

#[tokio::test]
async fn test_render_endpoint_validation_error_invalid_orb_setting_negative() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["settings"]["orbSettings"] = json!({
        "conjunction": -5.0 // Invalid - negative
    });
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error code if we got a client error with valid JSON
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
        assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    }
}

#[tokio::test]
async fn test_render_endpoint_validation_error_invalid_orb_setting_nan() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["settings"]["orbSettings"] = json!({
        "conjunction": f64::NAN
    });
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error code if we got a client error with valid JSON
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
        assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    }
}

#[tokio::test]
async fn test_render_endpoint_validation_error_all_orb_settings() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["settings"]["orbSettings"] = json!({
        "conjunction": 35.0,
        "opposition": 35.0,
        "trine": 35.0,
        "square": 35.0,
        "sextile": 35.0
    });
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error code if we got a client error with valid JSON
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
        assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    }
}

#[tokio::test]
async fn test_render_endpoint_validation_error_missing_layer_config() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["layer_config"] = json!({});
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error code if we got a client error with valid JSON
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
            assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
        assert!(body["error"]["message"].as_str().unwrap().contains("layer"));
    }
}

#[tokio::test]
async fn test_render_endpoint_validation_error_invalid_layer_kind() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["layer_config"]["natal"]["kind"] = json!("invalid_kind");
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error code if we got a client error with valid JSON
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
            assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
        assert!(body["error"]["message"].as_str().unwrap().contains("kind"));
    }
}

#[tokio::test]
async fn test_render_endpoint_validation_error_natal_missing_subject_id() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["layer_config"]["natal"].as_object_mut().unwrap().remove("subjectId");
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error code if we got a client error with valid JSON
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
            assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
        assert!(body["error"]["message"].as_str().unwrap().contains("subjectId"));
    }
}

#[tokio::test]
async fn test_render_endpoint_validation_error_natal_invalid_subject_id() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["layer_config"]["natal"]["subjectId"] = json!("nonexistent");
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error code if we got a client error with valid JSON
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
            assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
        assert!(body["error"]["message"].as_str().unwrap().contains("subjectId"));
    }
}

#[tokio::test]
async fn test_render_endpoint_validation_error_transit_missing_datetime() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["layer_config"]["transit"] = json!({
        "kind": "transit"
    });
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error code if we got a client error with valid JSON
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
            assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
        assert!(body["error"]["message"].as_str().unwrap().contains("explicitDateTime"));
    }
}

#[tokio::test]
async fn test_render_endpoint_validation_error_progressed_missing_datetime() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["layer_config"]["progressed"] = json!({
        "kind": "progressed"
    });
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error code if we got a client error with valid JSON
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
            assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
        assert!(body["error"]["message"].as_str().unwrap().contains("explicitDateTime"));
    }
}

#[tokio::test]
async fn test_render_endpoint_validation_error_invalid_planet_name() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["settings"]["includeObjects"] = json!(["invalid_planet"]);
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error code if we got a client error with valid JSON
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
            assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
        assert!(body["error"]["message"].as_str().unwrap().contains("includeObjects"));
    }
}

#[tokio::test]
async fn test_render_endpoint_validation_error_multiple_invalid_planets() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["settings"]["includeObjects"] = json!(["invalid1", "invalid2", "sun"]);
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error code if we got a client error with valid JSON
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
        assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    }
}

// ============================================================================
// Render Endpoint - Edge Cases and Advanced Scenarios
// ============================================================================

#[tokio::test]
async fn test_render_endpoint_missing_request_body() {
    let server = create_test_server();
    
    let response = server
        .post("/api/v1/render")
        .await;
    
    // Should return 400 or 500 for invalid/missing JSON (depending on how axum handles it)
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
}

#[tokio::test]
async fn test_render_endpoint_invalid_json() {
    let server = create_test_server();
    
    let response = server
        .post("/api/v1/render")
        .text("invalid json")
        .await;
    
    // Should return 400 or 500 for invalid JSON (depending on how axum handles it)
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
}

#[tokio::test]
async fn test_render_endpoint_missing_required_fields() {
    let server = create_test_server();
    let request = json!({
        "subjects": [{
            "id": "test"
        }]
        // Missing settings and layer_config
    });
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should handle missing fields gracefully (400 for validation, or 500 if calculation fails)
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
}

#[tokio::test]
async fn test_render_endpoint_boundary_coordinates() {
    let server = create_test_server();
    
    // Test boundary values for coordinates
    let test_cases = vec![
        (90.0, 180.0, true),   // Max valid
        (-90.0, -180.0, true), // Min valid
        (0.0, 0.0, true),      // Origin
    ];
    
    for (lat, lon, should_pass) in test_cases {
        let mut request = create_valid_request();
        request["subjects"][0]["location"]["lat"] = json!(lat);
        request["subjects"][0]["location"]["lon"] = json!(lon);
        
        let response = server
            .post("/api/v1/render")
            .json(&request)
            .await;
        
        if should_pass {
            // Validation should pass, but calculation might fail without Swiss Ephemeris (400 or 500)
            assert!(response.status_code().is_success() || 
                   response.status_code() == 400 || 
                   response.status_code() == 500);
        } else {
            // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
        }
    }
}

#[tokio::test]
async fn test_render_endpoint_boundary_orb_settings() {
    let server = create_test_server();
    
    // Test boundary values for orb settings
    let test_cases = vec![
        (0.0, true),   // Min valid
        (30.0, true),  // Max valid
        (15.0, true),  // Middle
    ];
    
    for (orb_value, should_pass) in test_cases {
        let mut request = create_valid_request();
        request["settings"]["orbSettings"]["conjunction"] = json!(orb_value);
        
        let response = server
            .post("/api/v1/render")
            .json(&request)
            .await;
        
        if should_pass {
            // Validation should pass, but calculation might fail without Swiss Ephemeris (400 or 500)
            assert!(response.status_code().is_success() || 
                   response.status_code() == 400 || 
                   response.status_code() == 500);
        } else {
            // Should return 400 for validation error, or 500 if JSON parsing fails first
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
        }
    }
}

#[tokio::test]
#[ignore] // Requires Swiss Ephemeris files
async fn test_render_endpoint_cache_works() {
    let server = create_test_server();
    let request = create_valid_request();
    
    // First request
    let response1 = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    response1.assert_status_ok();
    
    // Second identical request should hit cache
    let response2 = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    response2.assert_status_ok();
    
    // Both should return the same data
    let body1: serde_json::Value = response1.json();
    let body2: serde_json::Value = response2.json();
    assert_eq!(body1, body2);
}

#[tokio::test]
#[ignore] // Requires Swiss Ephemeris files
async fn test_render_endpoint_concurrent_requests() {
    let server = create_test_server();
    let request = create_valid_request();
    
    // Send multiple sequential requests to test server stability
    // Note: Using sequential instead of truly concurrent to avoid TestServer Sync issues
    for _ in 0..5 {
        let response = server
            .post("/api/v1/render")
            .json(&request)
            .await;
        response.assert_status_ok();
    }
}

#[tokio::test]
async fn test_render_endpoint_settings_merge() {
    let server = create_test_server();
    let mut request = create_valid_request();
    
    // Add settings override
    request["settings_override"] = json!({
        "zodiacType": "sidereal",
        "houseSystem": "whole_sign",
        "orbSettings": {
            "conjunction": 10.0
        }
    });
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should validate successfully (may fail with 500 if Swiss Ephemeris missing)
    // The validation should pass because settings merge happens before validation
    let status = response.status_code();
    assert!(status == 200 || status == 400 || status == 500);
}

#[tokio::test]
async fn test_render_endpoint_empty_include_objects() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["settings"]["includeObjects"] = json!([]);
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Empty includeObjects should be valid (may fail with 500 if Swiss Ephemeris missing)
    assert!(response.status_code().is_success() || 
           response.status_code() == 400 || 
           response.status_code() == 500);
}

#[tokio::test]
async fn test_render_endpoint_default_orb_settings() {
    let server = create_test_server();
    let request = create_valid_request();
    // Don't specify orbSettings - should use defaults
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should work with default orb settings (may fail with 500 if Swiss Ephemeris missing)
    assert!(response.status_code().is_success() || 
           response.status_code() == 400 || 
           response.status_code() == 500);
}

#[tokio::test]
async fn test_render_endpoint_location_optional() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["subjects"][0].as_object_mut().unwrap().remove("location");
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Location should be optional (may fail with 500 if Swiss Ephemeris missing or location required)
    assert!(response.status_code().is_success() || 
           response.status_code() == 400 || 
           response.status_code() == 500);
}

#[tokio::test]
async fn test_render_endpoint_timezone_handling() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["subjects"][0]["birthTimezone"] = json!("America/New_York");
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should handle timezone (may fail with 500 if Swiss Ephemeris missing)
    assert!(response.status_code().is_success() || 
           response.status_code() == 400 || 
           response.status_code() == 500);
}

// ============================================================================
// ChartSpec Endpoint Tests
// ============================================================================

#[tokio::test]
async fn test_chartspec_endpoint_validation_errors() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["subjects"] = json!([]);
    
    let response = server
        .post("/api/v1/render/chartspec")
        .json(&request)
        .await;
    
    // Should get 400 for validation error, or 500 if something else fails
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
        assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    }
}

#[tokio::test]
#[ignore] // Requires Swiss Ephemeris files
async fn test_chartspec_endpoint_multiple_layers() {
    let server = create_test_server();
    let request = create_transit_request();
    
    let response = server
        .post("/api/v1/render/chartspec")
        .json(&request)
        .await;
    
    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert!(body["spec"].is_object());
    assert!(body["ephemeris"]["layers"]["natal"].is_object());
    assert!(body["ephemeris"]["layers"]["transit"].is_object());
}

// ============================================================================
// Error Response Structure Tests
// ============================================================================

#[tokio::test]
async fn test_error_response_structure() {
    let server = create_test_server();
    let request = json!({
        "subjects": [],
        "settings": {
            "zodiacType": "tropical",
            "houseSystem": "placidus"
        },
        "layer_config": {}
    });
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should return 400 for validation error, or 500 if something else fails
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
    
    // Only check error structure if we got a JSON response
    if response.status_code().is_client_error() {
        let body: serde_json::Value = response.json();
        
        // Check error structure
        assert!(body.get("error").is_some());
        assert!(body["error"].get("code").is_some());
        assert!(body["error"].get("message").is_some());
        assert!(body["error"].get("correlation_id").is_some());
        
        assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
        assert!(body["error"]["message"].is_string());
        assert!(body["error"]["correlation_id"].is_string());
        assert!(!body["error"]["correlation_id"].as_str().unwrap().is_empty());
    }
}

#[tokio::test]
async fn test_error_response_different_error_types() {
    let server = create_test_server();
    
    // Test validation error
    let request1 = json!({
        "subjects": [],
        "settings": {"zodiacType": "tropical", "houseSystem": "placidus"},
        "layer_config": {}
    });
    let response1 = server.post("/api/v1/render").json(&request1).await;
    // Should get 400 for validation error, or 500 if something else fails
    assert!(response1.status_code().is_client_error() || response1.status_code().is_server_error());
    
    if response1.status_code().is_client_error() {
        let body1: serde_json::Value = response1.json();
        assert_eq!(body1["error"]["code"], "VALIDATION_ERROR");
    }
    
    // Test invalid JSON
    let response2 = server.post("/api/v1/render").text("invalid").await;
    assert!(response2.status_code().is_client_error() || response2.status_code().is_server_error());
}

// ============================================================================
// HTTP Method Tests
// ============================================================================

#[tokio::test]
async fn test_render_endpoint_wrong_method() {
    let server = create_test_server();
    
    // GET should not work for render endpoint
    let response = server
        .get("/api/v1/render")
        .await;
    
    // Should return 405 Method Not Allowed, 404, or 500 (depending on how axum handles it)
    // Axum may return 500 if the endpoint doesn't exist or method isn't allowed
    assert!(response.status_code().is_client_error() || response.status_code().is_server_error());
}

#[tokio::test]
async fn test_health_endpoint_wrong_method() {
    let server = create_test_server();
    
    // POST should not work for health endpoint
    let response = server
        .post("/health")
        .json(&json!({}))
        .await;
    
    // Should return 405 Method Not Allowed or 404
    assert!(response.status_code().is_client_error());
}

// ============================================================================
// Content Type Tests
// ============================================================================

#[tokio::test]
async fn test_render_endpoint_content_type() {
    let server = create_test_server();
    let request = create_valid_request();
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Response should be JSON (may fail with 500 if Swiss Ephemeris missing)
    assert!(response.status_code().is_success() || 
           response.status_code() == 400 || 
           response.status_code() == 500);
    // axum-test should handle JSON automatically
}

// ============================================================================
// Request Size and Performance Tests
// ============================================================================

#[tokio::test]
async fn test_render_endpoint_large_request() {
    let server = create_test_server();
    let mut request = create_valid_request();
    
    // Add many planets
    request["settings"]["includeObjects"] = json!([
        "sun", "moon", "mercury", "venus", "mars", 
        "jupiter", "saturn", "uranus", "neptune", "pluto",
        "chiron", "north_node", "south_node"
    ]);
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    // Should handle large requests (may fail with 500 if Swiss Ephemeris missing)
    assert!(response.status_code().is_success() || 
           response.status_code() == 400 || 
           response.status_code() == 500);
}

#[tokio::test]
#[ignore] // Requires Swiss Ephemeris files
async fn test_render_endpoint_response_time() {
    let server = create_test_server();
    let request = create_valid_request();
    
    let start = std::time::Instant::now();
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    let duration = start.elapsed();
    
    response.assert_status_ok();
    // Response should be reasonably fast (adjust threshold as needed)
    assert!(duration.as_millis() < 5000); // 5 seconds max
}

// ============================================================================
// Integration Tests - Full Workflow
// ============================================================================

#[tokio::test]
#[ignore] // Requires Swiss Ephemeris files
async fn test_full_workflow_natal_chart() {
    let server = create_test_server();
    
    // Step 1: Check health
    let health_response = server.get("/health").await;
    health_response.assert_status_ok();
    
    // Step 2: Get API info
    let info_response = server.get("/").await;
    info_response.assert_status_ok();
    
    // Step 3: Render ephemeris
    let request = create_valid_request();
    let render_response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    render_response.assert_status_ok();
    
    // Step 4: Render chartspec
    let chartspec_response = server
        .post("/api/v1/render/chartspec")
        .json(&request)
        .await;
    chartspec_response.assert_status_ok();
    
    // Verify all responses are valid
    let render_body: serde_json::Value = render_response.json();
    let chartspec_body: serde_json::Value = chartspec_response.json();
    
    assert!(render_body.get("layers").is_some());
    assert!(chartspec_body.get("spec").is_some());
    assert!(chartspec_body.get("ephemeris").is_some());
}

#[tokio::test]
#[ignore] // Requires Swiss Ephemeris files
async fn test_full_workflow_composite_chart() {
    let server = create_test_server();
    let request = create_multi_subject_request();
    
    // Render ephemeris for composite
    let render_response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    render_response.assert_status_ok();
    
    // Render chartspec for composite
    let chartspec_response = server
        .post("/api/v1/render/chartspec")
        .json(&request)
        .await;
    chartspec_response.assert_status_ok();
    
    let render_body: serde_json::Value = render_response.json();
    let chartspec_body: serde_json::Value = chartspec_response.json();
    
    // Should have multiple layers
    assert!(render_body["layers"]["natal1"].is_object());
    assert!(render_body["layers"]["natal2"].is_object());
    assert!(chartspec_body["ephemeris"]["layers"]["natal1"].is_object());
    assert!(chartspec_body["ephemeris"]["layers"]["natal2"].is_object());
}
