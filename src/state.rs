use reqwest::Client;
use std::env;

#[derive(Clone)]
pub struct PhixivState {
    pub client: Client,
}

impl PhixivState {
    pub async fn login() -> anyhow::Result<Self> {
        let verbose = env::var("TRACE_CLIENT_NETWORK")
            .unwrap_or_else(|_| String::from("false"))
            == "true";

        let client = Client::builder()
            .connection_verbose(verbose)
            .build()?;

        Ok(Self { client })
    }
}
