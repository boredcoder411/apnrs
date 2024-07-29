# apnrs

`apnrs` is a Rust library for sending push notifications to Apple devices using the Apple Push Notification service (APNs).

This crate provides utilities for creating the required payloads and sending the push notifications. 

## Usage

```rust
extern crate apnrs;
use apnrs::{send_push_notification, ApnsPayload, Aps};

#[tokio::main]
async fn main() {
    let payload = ApnsPayload {
        aps: Aps {
            alert: "Hello, world!".to_string(),
            content_available: 1,
            badge: Some(1),
            sound: Some("default".to_string()),
            category: None,
            thread_id: None,
        },
        custom_key: Some("custom_value".to_string()),
    };

    let response = send_push_notification(
        "path/to/auth/key",
        "TEAM_ID",
        "KEY_ID",
        "DEVICE_TOKEN",
        "com.example.app",
        payload,
        true
    ).await;

    match response {
        Ok(res) => println!("Notification sent: {:?}", res),
        Err(e) => eprintln!("Error sending notification: {:?}", e),
    }
}
```

## License

This project is licensed under the [MIT License](LICENSE).
