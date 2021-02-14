use crate::config;
use crate::pushover;

pub async fn report(provider: &str, message: String, config: &config::Config) {
    let e = format!("[{}] {}", provider, message);
    eprintln!("{}", e);

    if config.debug.unwrap() {
        pushover::send("Vaccinate Error".to_string(), e, config.clone()).await
    }
}
