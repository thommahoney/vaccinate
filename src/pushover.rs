use reqwest;
use serde::Serialize;

use crate::config;

// curl -X POST -H Content-Type:application/json \
//      -d '{"token": "...", "user": "...", "message": "...", "device": "...", "title": "...", "url": "..."}' \
//      https://api.pushover.net/1/messages.json

const PUSHOVER_URL: &'static str = "https://api.pushover.net/1/messages.json";

#[derive(Serialize)]
struct PushoverMessage {
    device: Option<String>,
    message: String,
    title: String,
    token: String,
    user: String,
}

pub async fn send(title: String, message: String, config: config::Config) {
    let pushover_config = config.pushover.unwrap();

    let message = PushoverMessage {
        device: pushover_config.device,
        message: message,
        title: title,
        token: pushover_config.app_token,
        user: pushover_config.user_token,
    };

    let pushover_json = serde_json::to_string(&message).unwrap();

    let client = reqwest::Client::new();
    let resp = client
        .post(PUSHOVER_URL)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json; charset=UTF-8")
        .body(pushover_json)
        .send()
        .await;

    if config.debug.unwrap() {
        match resp {
            Ok(resp) => {
                let status = resp.status();
                let text = resp.text().await.unwrap();
                eprintln!("[debug] Pushover status = {}, response = {}", status, text);
            }
            Err(e) => {
                eprintln!("[debug] Pushover error = {:?}", e);
            }
        }
    }
}
