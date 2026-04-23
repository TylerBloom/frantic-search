use anyhow::bail;
#[cfg(feature = "writable")]
use chrono::{DateTime, Utc};
use reqwest::Client;

pub struct FranticClient<T> {
    client: Client,
    marker: T,
}

/// A marker type used for [`FranticClient`] to mark the client as having read-only permissions.
pub struct ReadOnly();

/// A marker type used for [`FranticClient`] to mark the client as having admin permissions.
#[cfg(feature = "writable")]
pub struct Admin(String);

#[derive(Debug, Default)]
pub struct CrDocument {
    pub text: String,
    pub date: String,
}

impl FranticClient<ReadOnly> {
    pub fn connect() -> FranticClient<ReadOnly> {
        FranticClient {
            client: Client::new(),
            marker: ReadOnly(),
        }
    }
}

#[cfg(feature = "writable")]
impl FranticClient<Admin> {
    /// The path needs to point to an admin JWT.
    pub async fn connect_with_cred(
        path: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<FranticClient<Admin>> {
        use std::time::{SystemTime, UNIX_EPOCH};

        use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};

        #[derive(serde::Deserialize)]
        struct ServiceAccount {
            client_email: String,
            private_key: String,
            token_uri: String,
        }

        #[derive(serde::Serialize)]
        struct JwtClaims {
            iss: String,
            scope: String,
            aud: String,
            iat: u64,
            exp: u64,
        }

        let creds_json = std::fs::read_to_string(path)?;
        let service_account: ServiceAccount = serde_json::from_str(&creds_json)?;
        let client = Client::new();

        // Build and sign a JWT for the service account
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let claims = JwtClaims {
            iss: service_account.client_email,
            scope: "https://www.googleapis.com/auth/datastore".to_string(),
            aud: service_account.token_uri.clone(),
            iat: now,
            exp: now + 3600,
        };
        let encoding_key = EncodingKey::from_rsa_pem(service_account.private_key.as_bytes())?;
        let jwt = encode(&Header::new(Algorithm::RS256), &claims, &encoding_key)?;

        // Exchange the JWT for a Google OAuth2 access token
        let token_resp: serde_json::Value = client
            .post(&service_account.token_uri)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", jwt.as_str()),
            ])
            .send()
            .await?
            .json()
            .await?;
        let token = token_resp["access_token"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to get access_token"))?;

        Ok(FranticClient {
            client,
            marker: Admin(token.into()),
        })
    }
}

static FIREBASE_URL: &str = "https://firestore.googleapis.com/v1";
static PARENT: &str = "projects/applied-might-492316-v6/databases/frantic-search-fire/documents";

impl<T> FranticClient<T> {
    /// Unlike the `fetch_latest` method, this method first fetches the URL to the most recent CR.
    /// Then, fetches the CR from that URL.
    pub async fn fetch_latest_indirect(&self) -> anyhow::Result<CrDocument> {
        let resp = self
            .client
            .get(format!("{FIREBASE_URL}/{PARENT}/links/latest_cr"))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to fetch document: {}\n{}",
                resp.status(),
                resp.text().await?
            ));
        }

        let value: serde_json::Value = resp.json().await?;
        let document = &value["fields"];
        let text = &document["url"];
        let url = text["stringValue"].as_str().unwrap();

        let (_, date) = url.split_once(' ').unwrap();
        let (date, _) = date.split_once('.').unwrap();

        let year = &date[0..4];
        let month = match &date[4..6] {
            "01" => "January",
            "02" => "Febuary",
            "03" => "March",
            "04" => "April",
            "05" => "May",
            "06" => "June",
            "07" => "July",
            "08" => "August",
            "09" => "September",
            "10" => "October",
            "11" => "November",
            "12" => "December",
            month => bail!("Invalid date: {date:?}... {month:?}"),
        }
        .to_owned();
        let day = &date[6..8];

        let text = self.client.get(url).send().await?.text().await?;

        println!("{text:?}");

        Ok(CrDocument {
            text,
            date: format!("{month} {day}, {year}"),
        })
    }

    pub async fn fetch_latest(&self) -> anyhow::Result<CrDocument> {
        let resp = self
            .client
            .get(format!("{FIREBASE_URL}/{PARENT}/rules/latest"))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to fetch document: {}\n{}",
                resp.status(),
                resp.text().await?
            ));
        }

        let value: serde_json::Value = resp.json().await?;
        let document = &value["fields"];
        let text = &document["text"];
        let text = text["stringValue"].as_str().unwrap().into();
        let date = &document["date"];
        let date = date["timestampValue"].as_str().unwrap().to_string();
        let (date, _) = date.split_once("T").unwrap();
        let year = &date[0..4];
        let month = match &date[5..7] {
            "01" => "January",
            "02" => "Febuary",
            "03" => "March",
            "04" => "April",
            "05" => "May",
            "06" => "June",
            "07" => "July",
            "08" => "August",
            "09" => "September",
            "10" => "October",
            "11" => "November",
            "12" => "December",
            month => {
                return Err(anyhow::anyhow!(format!(
                    "Invalid date: {date:?}... {month:?}"
                )));
            }
        }
        .to_owned();
        let day = &date[8..10];
        Ok(CrDocument {
            text,
            date: format!("{month} {day}, {year}"),
        })
    }
}

#[cfg(feature = "writable")]
impl FranticClient<Admin> {
    pub async fn write(&self, text: String, date: DateTime<Utc>) -> anyhow::Result<()> {
        let write_resp = self
            .client
            .post(format!("{FIREBASE_URL}/{PARENT}/rules"))
            .header("Authorization", format!("Bearer {}", self.marker.0))
            .json(&serde_json::json!({
                "fields": {
                    "text": { "stringValue": text },
                    "date": { "timestampValue": date }
                }
            }))
            .send()
            .await?;

        if !write_resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to write document: {}\n{}",
                write_resp.status(),
                write_resp.text().await?
            ));
        }

        Ok(())
    }

    pub async fn write_latest_rules(
        &self,
        text: String,
        date: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        use chrono::Datelike;

        let write_resp = self
            .client
            .patch(format!("{FIREBASE_URL}/{PARENT}/rules/latest"))
            .header("Authorization", format!("Bearer {}", self.marker.0))
            .json(&serde_json::json!({
                "fields": {
                    "text": { "stringValue": text },
                    "date": { "timestampValue": date }
                }
            }))
            .send()
            .await?;

        if !write_resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to write document: {}\n{}",
                write_resp.status(),
                write_resp.text().await?
            ));
        }

        let url = format!(
            "https://media.wizards.com/{0}/downloads/MagicCompRules {0}{1:0>2}{2:0>2}.txt",
            date.year(),
            date.month(),
            date.day(),
        );
        let write_resp = self
            .client
            .patch(format!(
                "{FIREBASE_URL}/{PARENT}/links/latest_cr"
            ))
            .header("Authorization", format!("Bearer {}", self.marker.0))
            .json(&serde_json::json!({
                "fields": {
                    "url": { "stringValue": url },
                }
            }))
            .send()
            .await?;

        if !write_resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to write document: {}\n{}",
                write_resp.status(),
                write_resp.text().await?
            ));
        }

        Ok(())
    }

    pub async fn write_to_log(&self) -> anyhow::Result<()> {
        use chrono::Utc;

        let date = Utc::now();

        let write_resp = self
            .client
            .post(format!("{FIREBASE_URL}/{PARENT}/update_logs"))
            .header("Authorization", format!("Bearer {}", self.marker.0))
            .json(&serde_json::json!({
                "fields": {
                    "date": { "timestampValue": date }
                }
            }))
            .send()
            .await?;

        if !write_resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to write document: {}\n{}",
                write_resp.status(),
                write_resp.text().await?
            ));
        }
        Ok(())
    }

    pub async fn fetch_last_update_log(&self) -> anyhow::Result<DateTime<Utc>> {
        let body = r#"{
        "structuredQuery": {
            "from": [{"collectionId": "update_logs"}],
            "orderBy": [{ "field": { "fieldPath": "date" }, "direction": "Descending" }],
            "limit": 1
        }
    }"#;
        let resp = self
            .client
            .post(format!("{FIREBASE_URL}/{PARENT}:runQuery"))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        if !resp.status().is_success() {
            Err(anyhow::anyhow!(
                "Failed to fetch document: {}\n{}",
                resp.status(),
                resp.text().await?
            ))
        } else {
            use std::str::FromStr;

            let value: serde_json::Value = resp.json().await?;
            let document = &value[0]["document"]["fields"];
            let date = document["date"]["timestampValue"].as_str().unwrap();
            Ok(DateTime::<Utc>::from_str(date).unwrap())
        }
    }
}
