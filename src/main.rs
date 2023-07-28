use tumblr_api::{auth::read_credentials, TumblrClient};

const CLIENT_CACHE_PATH: &str = "client.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let credentials = read_credentials()?;
    let tumblr_client = TumblrClient::try_from_file_or_authorize(
        CLIENT_CACHE_PATH.into(),
        credentials,
        reqwest::Client::new(),
    )
    .await?;
    tumblr_client.save_to_file(CLIENT_CACHE_PATH.into())?;
    Ok(())
}
