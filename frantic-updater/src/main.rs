use chrono::{Datelike, Days, Utc};
use frantic_client::FranticClient;
use reqwest::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cred_path = std::env::var("FIREBASE_CREDS")?;
    let fs_client = FranticClient::connect_with_cred(cred_path).await.unwrap();
    let now = Utc::now();

    // Get the date that this was last ran. There will be a logs collection.
    let mut date = fs_client.fetch_last_update_log().await.unwrap();

    // Loop until we can WOTC for today's rules.
    let client = Client::new();
    loop {
        println!("Checking rules for {date}");
        let url = format!(
            "https://media.wizards.com/{0}/downloads/MagicCompRules {0}{1:0>2}{2:0>2}.txt",
            date.year(),
            date.month(),
            date.day(),
        );
        let resp = client.get(url).send().await.unwrap();
        if resp.status().is_success() {
            println!("Found CR text from {date}");
            let text = resp.text().await.unwrap();
            let text = frantic_core::normalize_cr_text(&text);
            fs_client.write(text, date).await.unwrap();
        }

        date = date + Days::new(1);
        if date > now {
            break;
        }
    }

    println!("Writing to update log");
    fs_client.write_to_log().await
}
