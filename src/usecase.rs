use std::collections::HashMap;
use std::ops::Sub;
use chrono::Duration;
use teloxide::adaptors::AutoSend;
use teloxide::types::ChatId;
use teloxide::Bot;
use serde::{Serialize, Deserialize};
use chrono::prelude::*;

pub struct TgBot<'a> {
    pub bot: &'a AutoSend<Bot>,
    pub usecase: UseCase,
    pub token: String,
    pub telegram_id: i64
}

pub struct UseCase {}

impl<'a> TgBot<'a> {
    pub fn new(bot: &'a AutoSend<Bot>, usecase: UseCase, token: String, telegram_id: i64) -> TgBot<'a> {
        TgBot { bot, usecase, token, telegram_id }
    }

    pub fn start(&self) -> String {
        "<b>Welcome to Feed Fly, your Feedly assistant.</b>
You can contact me at <a href=\"linbuxiao@gmail.com\"><i>linbuxiao@gmail.com</i></a> with your thoughts.
Now to get your <a href=\"https://feedly.com/v3/auth/dev\">token</a> and set to me by <code>/token</code> !".to_string()
    }

    pub async fn list(&self) -> String {
        match self.usecase.get_collection_list(self.token.to_string()).await {
            Ok(result) => {
                log::info!("{:?}", result);
                let mut render_str = "".to_string();
                for (label, feeds) in result {
                    let mut all = format!("<b>{}</b>\n\n", label);
                    for feed in feeds {
                        all += &format!("<a href=\"{href}\">{title}</a>\n", href=feed.href, title=feed.title).to_string();
                    }
                    all += "\n";
                    render_str += &all;
                }
                log::info!("{}", render_str);
                render_str.to_string()
            }
            Err(e) => e.to_string()
        }
    }
}

impl UseCase {
    pub fn new() -> Self { UseCase {  } }
    async fn get_collection_list(&self, token: String) -> Result<HashMap<String, Vec<CollectionRo>>, reqwest::Error> {
        let client = reqwest::Client::new();
        let mut  result = client.get("https://cloud.feedly.com/v3/collections").bearer_auth(&token).send().await?;
        let collection_list = result.json::<Vec<Collection>>().await?;
        let mut entry_ids:Vec<String> = Vec::new();
        let mut entry_label_map: HashMap<String, String> = HashMap::new();
        for v in collection_list {
            result = client.get(format!("https://cloud.feedly.com/v3/streams/ids?streamId={}", v.id)).bearer_auth(&token).send().await?;
            let mut ids = result.json::<EntryIdsResponse>().await?.ids;
            for s in &ids {
                entry_label_map.insert(s.to_string(), v.label.to_string());
            }
            entry_ids.append(&mut ids);
        }
        result = client.post("https://cloud.feedly.com/v3/entries/.mget").bearer_auth(&token).json(&entry_ids).send().await?;
        let entry_list = result.json::<Vec<EntryResponse>>().await?;
        let mut label_entry_map: HashMap<String, Vec<CollectionRo>> = HashMap::new();
        for v in entry_list {
            let yesterday = Local::now().date().sub(Duration::days(1));
            let published_utc = Utc.timestamp_millis(v.published).date().naive_utc();
            let published = Local.from_utc_date(&published_utc);
            if published.le(&yesterday) {continue}
            let label = entry_label_map.get(&v.id).unwrap();
            if v.alternate.is_empty() { continue }
            let alternate = v.alternate[0].to_owned();
            let href = alternate.href.unwrap_or_default();

            if label_entry_map.contains_key(label) {
                let result = label_entry_map.get_mut(label).unwrap();
                result.push(CollectionRo{
                    title: v.title,
                    href
                });
            } else {
                let wait_use_result:Vec<CollectionRo> = vec![CollectionRo{
                    title: v.title,
                    href
                }];
                label_entry_map.insert(label.to_string(), wait_use_result);
            }
        }
        Ok(label_entry_map)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CollectionRo {
    title: String,
    href: String
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Collection {
    id: String,
    label: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EntryIdsResponse {
    ids: Vec<String>,
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EntryResponse {
    id: String,
    title: String,
    published: i64,
    alternate: Vec<Alternate>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Alternate {
    href: Option<String>,
    #[serde(rename = "type")]
    type_field: Option<String>,
}
