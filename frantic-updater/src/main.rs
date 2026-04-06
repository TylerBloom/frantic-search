use chrono::{Datelike, Utc};
use frantic_client::FranticClient;
use reqwest::Client;
use time::{Date, Month};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let fs_client = FranticClient::connect().await.unwrap();
    // Make sure the DB is in an expected state and that the prior run was not a failure
    // TODO: Act on the errors if any
    fs_client.health_check().await.unwrap();

    let now = Utc::now();
    let now = Date::from_ordinal_date(now.year(), now.ordinal() as u16).unwrap();

    let mut date = match fs_client.fetch_latest_cr_and_metadata().await.unwrap() {
        Some((_, prior_date, _)) => prior_date,
        // This means the DB has no rules. To find rules, we'll start from the beginning of the
        // year. This can still cause problems, but its good enough for now...
        None => Date::from_calendar_date(now.year(), Month::January, 1).unwrap(),
    };

    let client = Client::new();
    loop {
        let url = format!(
            "https://media.wizards.com/{0}/downloads/MagicCompRules {0}{1:0>2}{2:0>2}.txt",
            date.year(),
            date.month() as u8,
            date.day(),
        );
        let resp = client.get(url).send().await.unwrap();
        if resp.status().is_success() {
            println!("Found CR text from {date}");
            let text = resp.text().await.unwrap();
            let text = frantic_core::normalize_cr_text(&text);
            fs_client.insert_new_cr(text, date).await.unwrap();
        }

        date = date.next_day().unwrap();
        if date > now {
            break Ok(());
        }
    }
}
