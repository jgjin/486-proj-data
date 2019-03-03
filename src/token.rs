use reqwest::{
    Client,
};
use serde::{
    Deserialize,
    Serialize,
};
use std::{
    collections::{
        HashMap,
    },
};

#[derive(Debug, Deserialize, Serialize)]
pub struct AccessToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i32,
    pub scope: String,
}

pub fn retrieve_access_token(
    client: &Client,
) -> Result<AccessToken, reqwest::Error> {
    let client_id = "480d0d13691a4a1eb10363baddc3c3d4";
    let client_secret = "***REMOVED***";

    let mut form_data = HashMap::new();
    form_data.insert("grant_type", "client_credentials");

    Ok(
     client.post("https://accounts.spotify.com/api/token/")
        .basic_auth(client_id, Some(client_secret))
        .form(&form_data)
        .send()?
        .json()?
    )
}

