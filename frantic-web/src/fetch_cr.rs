use chrono::{DateTime, Utc};
use frantic_client::{CrDocument, FranticClient};
use serde::{Deserialize, Serialize};
use web_sys::Storage;

use crate::Msg;

trait CachedItem: Serialize + Deserialize {
    const KEY: &str;

    fn get_cache(storage: &Storage) -> Option<Self> {
        storage
            .get_item(Self::KEY)
            .and_then(|value| serde_json::from_str(&value).ok())
    }

    fn set_cache(&self, storage: &Storage) {
        storage.set_item(Self::KEY, &serde_json::to_string(self));
    }
}

impl CachedItem for CrDocument {
    const KEY: &str = "frantic_cr";
}

impl CachedItem for DateTime<Utc> {
    const KEY: &str = "frantic_cr_date";
}

pub async fn fetch_cr() -> Msg {
    let Some(storage) = web_sys::window().and_then(|win| win.local_storage().ok().flatten()) else {
        return Msg::Cr(None);
    };
    let client = FranticClient::connect();
    if !check_cache_expiration(&storage, &client).await {
        return Msg::Cr(None);
    }

    let client = frantic_client::FranticClient::connect();
    let cr = match client.fetch_latest().await {
        Ok(cr) => cr,
        Err(err) => {
            gloo::console::log!(err.to_string());
            panic!()
        }
    };
    set_cached_cr(&cr);
    Msg::Cr(cr)
}

async fn check_cache_expiration<T>(storage: &Storage, client: &FranticClient<T>) -> bool {
    let Some(now) = DateTime::<Utc>::get_cache(storage) else {
        return false;
    };
    client
        .fetch_date_of_latest_cr()
        .await
        .is_ok_and(|then| now > then)
}

pub fn get_cached_cr(storage: &Storage) -> Option<frantic_client::CrDocument> {
    CrDocument::get_cache(storage)
}

fn set_cached_cr(cr: &CrDocument) {
}
