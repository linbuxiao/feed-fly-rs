use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Setting {
    pub bot_token: String,
    pub telegram_id: i64,
    pub feedly_token: String
}

pub async fn default_config() -> Setting {
    let config = tokio::fs::read_to_string("config.yaml").await.unwrap();
    serde_yaml::from_str(&config).unwrap()
}
