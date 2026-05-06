use frantic_client::{CrDocument, FranticClient};

static CR_CACHE_KEY: &str = "frantic_cr";

pub fn try_get_cached_cr() -> Option<CrDocument> {
    let storage = web_sys::window()?.local_storage().ok()??;
    let cached_cr: String = storage.get_item(CR_CACHE_KEY).ok()??;
    serde_json::from_str(&cached_cr).ok()
}

pub async fn get_latest_cr() -> CrDocument {
    let client = FranticClient::connect();
    if let Some(cr) = try_get_cached_cr() {
        let Ok(latest_date) = client.fetch_latest_date().await else {
            return CrDocument::default();
        };
        if latest_date <= cr.date {
            return cr;
        }
    }
    let latest_cr = client.fetch_latest().await.unwrap_or_default();
    if let Ok(json) = serde_json::to_string(&latest_cr) {
        let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok()?) else {
            return latest_cr;
        };
        let _ = storage.set_item(CR_CACHE_KEY, &json);
    }
    latest_cr
}
