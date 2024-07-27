extern crate jsonwebtoken as jwt;

use jwt::{encode, Header, EncodingKey};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub iss: String,
    pub iat: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Aps {
    pub alert: String,
    #[serde(rename = "content-available")]
    pub content_available: u8,
    pub badge: Option<u32>,
    pub sound: Option<String>,
    pub category: Option<String>,
    pub thread_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApnsPayload {
    pub aps: Aps,
    pub custom_key: Option<String>,
}

fn get_current_unix_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

pub async fn send_push_notification(
    auth_key_path: &str,
    team_id: &str,
    key_id: &str,
    device_token: &str,
    topic: &str,
    payload: ApnsPayload,
) -> Result<(), reqwest::Error> {
    // Read the key from file
    let key = fs::read_to_string(auth_key_path).expect("Unable to read file");

    // Create the JWT token
    let claims = Claims {
        iss: team_id.to_string(),
        iat: get_current_unix_time(),
    };

    let header = Header {
        alg: jwt::Algorithm::ES256,
        kid: Some(key_id.to_string()),
        ..Default::default()
    };

    let token = encode(&header, &claims, &EncodingKey::from_ec_pem(key.as_bytes()).unwrap()).unwrap();

    // Prepare the headers and body for the HTTP request
    let url = format!("https://api.sandbox.push.apple.com/3/device/{}", device_token);

    let body = serde_json::to_string(&payload).expect("Failed to serialize payload");

    let mut headers = HeaderMap::new();
    headers.insert("apns-topic", HeaderValue::from_str(topic).unwrap());
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("bearer {}", token)).unwrap());
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    // Create an HTTP/2 client and send the request
    let client = reqwest::Client::builder()
        .http2_prior_knowledge()
        .build()
        .expect("Failed to build client");

    println!("Sending request to APNs...");

    let response = client.post(&url)
        .headers(headers)
        .body(body)
        .send()
        .await?;

    // Print the response
    println!("Status: {}", response.status());
    println!("Headers: {:?}", response.headers());

    Ok(())
}
