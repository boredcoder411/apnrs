# apnrs

Other apns libraries in rust are annoyingly large, so I wrote this one

## Installation

Run this in your terminal:
```bash
cargo add apnrs
```
Or add this to your Cargo.toml:
```toml
[dependencies]
apnrs = "0.2.0"
```

## Usage

```rust
use apnrs::{send_push_notification, ApnsPayload, Aps};

#[tokio::main]
async fn main() {
    // Example values
    let auth_key_path = "path/to/AuthKey.p8";
    let team_id = "your_team_id";
    let key_id = "your_key_id";
    let device_token = "device_token";
    let topic = "your_topic";

    let aps = Aps {
        alert: "Hello, World!".to_string(),
        content_available: 1,
        badge: Some(1),
        sound: Some("default".to_string()),
        category: None,
        thread_id: None,
    };

    let payload = ApnsPayload {
        aps,
        custom_key: Some("custom_value".to_string()),
    };

    match send_push_notification(auth_key_path, team_id, key_id, device_token, topic, payload).await {
        Ok(_) => println!("Push notification sent successfully!"),
        Err(e) => eprintln!("Failed to send push notification: {}", e),
    }
}
```

## License

This project is licensed under the [MIT License](LICENSE).
