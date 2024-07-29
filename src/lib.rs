//! # apnrs
//!
//! `apnrs` is a Rust library for sending push notifications to Apple devices using the Apple Push Notification service (APNs).
//!
//! This crate provides utilities for creating the required payloads and sending the push notifications. 
//!
//! ## Examples
//!
//! Here is a basic example of how to use `apnrs` to send a push notification:
//!
//! ```rust,no_run
//! extern crate apnrs;
//! use apnrs::{send_push_notification, ApnsPayload, Aps};
//!
//! #[tokio::main]
//! async fn main() {
//!     let payload = ApnsPayload {
//!         aps: Aps {
//!             alert: "Hello, world!".to_string(),
//!             content_available: 1,
//!             badge: Some(1),
//!             sound: Some("default".to_string()),
//!             category: None,
//!             thread_id: None,
//!         },
//!         custom_key: Some("custom_value".to_string()),
//!     };
//!
//!     let response = send_push_notification(
//!         "path/to/auth/key",
//!         "TEAM_ID",
//!         "KEY_ID",
//!         "DEVICE_TOKEN",
//!         "com.example.app",
//!         payload,
//!         true
//!     ).await;
//!
//!     match response {
//!         Ok(res) => println!("Notification sent: {:?}", res),
//!         Err(e) => eprintln!("Error sending notification: {:?}", e),
//!     }
//! }
//! ```
//!
//! ## Structs
//! 
//! * [`ApnsPayload`](struct.ApnsPayload.html) - Represents the entire payload sent to the APNs.
//! * [`Aps`](struct.Aps.html) - Represents the APNs (Apple Push Notification service) payload.
//! * [`Claims`](struct.Claims.html) - Represents the claims used for generating the JWT token.
//!
//! ## Functions
//! 
//! * [`send_push_notification`](fn.send_push_notification.html) - Sends a push notification to an Apple device using APNs.

extern crate jsonwebtoken as jwt;

use jwt::{encode, EncodingKey, Header};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Response;
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents the claims used for generating the JWT token.
///
/// # Fields
///
/// * `iss` - The issuer of the token, typically your team ID.
/// * `iat` - The issued at time, specified as a Unix timestamp.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub iss: String,
    pub iat: u64,
}

/// Represents the APNs (Apple Push Notification service) payload.
///
/// # Fields
///
/// * `alert` - The alert message to be displayed.
/// * `content_available` - Indicates if new content is available (set to 1).
/// * `badge` - The number to display as the badge of the app icon.
/// * `sound` - The name of the sound file to play for an alert.
/// * `category` - The category of the notification.
/// * `thread_id` - The thread identifier for the notification.
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

/// Represents the entire payload sent to the APNs.
///
/// # Fields
///
/// * `aps` - The APS payload.
/// * `custom_key` - Any additional custom data to be sent with the notification.
#[derive(Debug, Serialize, Deserialize)]
pub struct ApnsPayload {
    pub aps: Aps,
    pub custom_key: Option<String>,
}

/// Retrieves the current Unix timestamp.
///
/// # Returns
///
/// The current time in seconds since the Unix epoch.
fn get_current_unix_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

/// Sends a push notification to an Apple device using APNs.
///
/// # Arguments
///
/// * `auth_key_path` - The path to the file containing the APNs auth key.
/// * `team_id` - Your Apple Developer team ID.
/// * `key_id` - The key ID associated with your APNs auth key.
/// * `device_token` - The device token of the target device.
/// * `topic` - The topic (usually the app's bundle ID) for the notification.
/// * `payload` - The payload of the notification.
/// * `prod` - A boolean indicating whether to use the production or sandbox environment.
///
/// # Returns
///
/// A `Result` containing either the HTTP response from the APNs server or a `reqwest::Error`.
///
/// # Example
///
/// ```rust,no_run
/// let payload = ApnsPayload {
///     aps: Aps {
///         alert: "Hello, world!".to_string(),
///         content_available: 1,
///         badge: Some(1),
///         sound: Some("default".to_string()),
///         category: None,
///         thread_id: None,
///     },
///     custom_key: Some("custom_value".to_string()),
/// };
///
/// let response = send_push_notification(
///     "path/to/auth/key",
///     "TEAM_ID",
///     "KEY_ID",
///     "DEVICE_TOKEN",
///     "com.example.app",
///     payload,
///     true
/// ).await;
///
/// match response {
///     Ok(res) => println!("Notification sent: {:?}", res),
///     Err(e) => eprintln!("Error sending notification: {:?}", e),
/// }
/// ```
pub async fn send_push_notification(
    auth_key_path: &str,
    team_id: &str,
    key_id: &str,
    device_token: &str,
    topic: &str,
    payload: ApnsPayload,
    prod: bool
) -> Result<Response, reqwest::Error> {
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

    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_ec_pem(key.as_bytes()).unwrap(),
    )
    .unwrap();

    // Prepare the headers and body for the HTTP request
    let url = if prod {
        format!(
            "https://api.push.apple.com/3/device/{}",
            device_token
        )
    } else {
        format!(
            "https://api.sandbox.push.apple.com/3/device/{}",
            device_token
        )
    };

    let body = serde_json::to_string(&payload).expect("Failed to serialize payload");

    let mut headers = HeaderMap::new();
    headers.insert("apns-topic", HeaderValue::from_str(topic).unwrap());
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("bearer {}", token)).unwrap(),
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    // Create an HTTP/2 client and send the request
    let client = reqwest::Client::builder()
        .http2_prior_knowledge()
        .build()
        .expect("Failed to build client");

    let response = client.post(&url).headers(headers).body(body).send().await?;

    Ok(response)
}
