use crate::util::{
	auth::DiscordUser,
	config::CONFIG
};
use std::collections::HashMap;
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use url::Url;

pub async fn me(token_type: &str, token: &str) -> Result<DiscordUser, Error> {
	let client = Client::new();

	let res = client.get("https://discord.com/api/users/@me").header("Authorization", format!("{} {}", token_type, token)).send().await;

	let res = match res {
		Ok(res) => res,
		Err(e) => return Err(e)
	};

	let json: Result<DiscordUser, _> = res.json().await;
	match json {
		Ok(json) => return Ok(json),
		Err(err) => return Err(err),
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub scope: String,
}

pub async fn fetch_access_token(code: &str) -> Result<AccessTokenResponse, Error> {
	let mut form: HashMap<&str, &str> = HashMap::new();
    form.insert("client_id", &CONFIG.discord_id);
    form.insert("client_secret", &CONFIG.discord_secret);
    form.insert("grant_type", "authorization_code");
    form.insert("code", code);
    form.insert("redirect_uri", &CONFIG.discord_redirect);

    let scopes = &CONFIG.discord_scopes.join(" ");
    form.insert("scope", scopes);

    let url = Url::parse("https://discord.com/api/oauth2/token").unwrap();
	let client = Client::new();
	
	let req = client.post(url).form(&form).send().await;

	let res = match req {
		Ok(res) => res,
		Err(e) => return Err(e)
	};

	let json: Result<AccessTokenResponse, _> = res.json().await;
	match json {
		Ok(json) => return Ok(json),
		Err(err) => return Err(err),
	}
}
