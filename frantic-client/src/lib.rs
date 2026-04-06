#[cfg(feature = "writable")]
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::Client;
use serde::{Deserialize, Serialize};

use chrono::Utc;

pub struct FranticClient<T> {
    client: Client,
    marker: T,
}

/// A marker type used for [`FranticClient`] to mark the client as having read-only permissions.
pub struct ReadOnly();

/// A marker type used for [`FranticClient`] to mark the client as having admin permissions.
pub struct Admin(String);

#[cfg(feature = "writable")]
#[derive(Deserialize)]
struct ServiceAccount {
    client_email: String,
    private_key: String,
    token_uri: String,
}

#[cfg(feature = "writable")]
#[derive(Serialize)]
struct JwtClaims {
    iss: String,
    scope: String,
    aud: String,
    iat: u64,
    exp: u64,
}

impl FranticClient<ReadOnly> {
    pub fn connect() -> FranticClient<ReadOnly> {
        FranticClient {
            client: Client::new(),
            marker: ReadOnly(),
        }
    }
}

impl FranticClient<Admin> {
    /// The path needs to point to an admin JWT.
    #[cfg(feature = "writable")]
    pub async fn connect_with_cred(path: impl AsRef<Path>) -> anyhow::Result<FranticClient<Admin>> {
        use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};

        let creds_json = std::fs::read_to_string(path).expect("Failed to read credentials file");
        let service_account: ServiceAccount =
            serde_json::from_str(&creds_json).expect("Failed to parse credentials file");
        let client = Client::new();

        // Build and sign a JWT for the service account
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let claims = JwtClaims {
            iss: service_account.client_email,
            scope: "https://www.googleapis.com/auth/datastore".to_string(),
            aud: service_account.token_uri.clone(),
            iat: now,
            exp: now + 3600,
        };
        let encoding_key = EncodingKey::from_rsa_pem(service_account.private_key.as_bytes())
            .expect("Failed to load private key");
        let jwt = encode(&Header::new(Algorithm::RS256), &claims, &encoding_key)
            .expect("Failed to sign JWT");

        // Exchange the JWT for a Google OAuth2 access token
        let token_resp: serde_json::Value = client
            .post(&service_account.token_uri)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", jwt.as_str()),
            ])
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let token = token_resp["access_token"]
            .as_str()
            .expect("Failed to get access_token");

        Ok(FranticClient {
            client,
            marker: Admin(token.into()),
        })
    }
}

#[derive(Debug, Default)]
pub struct CrDocument {
    pub text: String,
    pub date: String,
}

// pub type CrDocument = serde_json::Value;

impl<T> FranticClient<T> {
    pub async fn fetch_latest(&self) -> anyhow::Result<CrDocument> {
        let body = r#"{
        "structuredQuery": {
            "from": [{"collectionId": "rules"}],
            "orderBy": [{ "field": { "fieldPath": "date" }, "direction": "Descending" }],
            "limit": 1
        }
    }"#;
        let resp = self.client
        .post("https://firestore.googleapis.com/v1/projects/applied-might-492316-v6/databases/frantic-search-fire/documents:runQuery")
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .unwrap();
        assert!(resp.status().is_success());

        let value: serde_json::Value = resp.json().await?;
        // let document = &value;
        // let document = &value["document"];
        let document = &value[0]["document"]["fields"];
        Ok(CrDocument {
            text: document["text"]["stringValue"].as_str().unwrap().into(),
            date: document["date"]["timestampValue"].as_str().unwrap().into(),
        })
    }
}

#[cfg(feature = "writable")]
impl FranticClient<Admin> {
    pub async fn write(&self) -> anyhow::Result<()> {
        let write_resp = self.client
        .post("https://firestore.googleapis.com/v1/projects/applied-might-492316-v6/databases/frantic-search-fire/documents/rules")
        .header("Authorization", format!("Bearer {}", self.marker.0))
        .json(&serde_json::json!({
            "fields": {
                "text": { "stringValue": "New rule" },
                "date": { "timestampValue": Utc::now() }
            }
        }))
        .send()
        .await
        .unwrap();
        assert!(write_resp.status().is_success());
        Ok(())
    }
}

/*
impl FranticClient {
    async fn check_db_status(&self) -> anyhow::Result<()> {
        sqlx::query(
            r#"
CREATE TABLE IF NOT EXISTS rules (
  id uuid,
  release_date date,
  cr text
);
"#,
        )
        .execute(&self.pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
CREATE TABLE IF NOT EXISTS logs (
  run_date timestamptz,
  found_new_cr bool,
  log text
);
"#,
        )
        .execute(&self.pool)
        .await
        .unwrap();

        Ok(())
    }

    pub async fn fetch_latest_cr(&self) -> anyhow::Result<Option<String>> {
        self.fetch_latest_cr_and_metadata()
            .await
            .map(|row| row.map(|row| row.2))
    }

    pub async fn fetch_latest_cr_and_metadata(
        &self,
    ) -> anyhow::Result<Option<(Uuid, Date, String)>> {
        let result = sqlx::query_as(
            "
SELECT id, release_date, cr
FROM rules
ORDER BY release_date

",
        )
        .persistent(false)
        .fetch_one(&self.pool)
        .await;
        match result {
            Ok(row) => Ok(Some(row)),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn health_check(&self) -> anyhow::Result<Vec<Error>> {
        // TODO: Actual check things...
        Ok(Vec::new())
    }

    pub async fn insert_new_cr(&self, text: String, date: Date) -> anyhow::Result<()> {
        let id = Uuid::new_v4();
        let _result: (Uuid, Date, String) = sqlx::query_as(
            "
INSERT INTO
    rules ( id, release_date, cr )
VALUES
    ( $1, $2, $3 )
RETURNING
    id, release_date, cr
",
        )
        .bind(id)
        .bind(date)
        .bind(text)
        .persistent(true)
        .fetch_one(&self.pool)
        .await
        .unwrap();
        Ok(())
    }
}
*/
