use reqwest::Client;

#[derive(Clone)]
pub struct PhixivState {
    pub client: Client,
}

impl PhixivState {
    pub async fn login() -> anyhow::Result<Self> {
        let client = Client::new();

        Ok(Self { client })
    }
}
