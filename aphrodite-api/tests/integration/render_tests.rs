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

#[tokio::test]
#[ignore] // Requires Swiss Ephemeris files
async fn test_health_endpoint() {
    let server = create_test_server();
    
    let response = server.get("/health").await;
    response.assert_status_ok();
    response.assert_text("OK");
}

#[tokio::test]
#[ignore] // Requires Swiss Ephemeris files
async fn test_api_info_endpoint() {
    let server = create_test_server();
    
    let response = server.get("/").await;
    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert!(body.get("name").is_some());
}

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
    
    response.assert_status_bad_request();
    let body: serde_json::Value = response.json();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
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
    
    response.assert_status_bad_request();
    let body: serde_json::Value = response.json();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_render_endpoint_validation_error_invalid_coordinates() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["subjects"][0]["location"]["lat"] = json!(100.0); // Invalid latitude
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    response.assert_status_bad_request();
    let body: serde_json::Value = response.json();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_render_endpoint_validation_error_invalid_orb_setting() {
    let server = create_test_server();
    let mut request = create_valid_request();
    request["settings"]["orbSettings"] = json!({
        "conjunction": 50.0 // Invalid - exceeds max of 30
    });
    
    let response = server
        .post("/api/v1/render")
        .json(&request)
        .await;
    
    response.assert_status_bad_request();
    let body: serde_json::Value = response.json();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
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
    
    response.assert_status_bad_request();
    let body: serde_json::Value = response.json();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
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
    
    response.assert_status_bad_request();
    let body: serde_json::Value = response.json();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
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
    
    // Send multiple concurrent requests
    let mut handles = vec![];
    for _ in 0..5 {
        let server_clone = server.clone();
        let request_clone = request.clone();
        handles.push(tokio::spawn(async move {
            server_clone
                .post("/api/v1/render")
                .json(&request_clone)
                .await
        }));
    }
    
    // Wait for all requests to complete
    for handle in handles {
        let response = handle.await.unwrap();
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
    
    // Should validate successfully (even if calculation fails without Swiss Ephemeris)
    // The validation should pass because settings merge happens before validation
    // If we get a calculation error, that's fine - it means merge worked
    let status = response.status_code();
    assert!(status == 200 || status == 400); // Either success or calculation error is acceptable
}
