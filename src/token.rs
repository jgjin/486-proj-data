use std::{
    collections::{
        HashMap,
    },
    sync::{
        Arc,
    },
    thread::{
        self,
        sleep,
    },
    time::{
        Duration,
    },
};

use atomicring::{
    AtomicRingQueue,
};
use reqwest::{
    Client,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    io::{
        structs_from_file,
    },
};

#[derive(Debug, Deserialize, Serialize)]
struct AccessToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i32,
    pub scope: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct SpotifyClient {
    pub name: String,
    pub id: String,
    pub secret: String,
}

struct SpotifyClientWithToken {
    pub token: String,
    pub client: SpotifyClient,
}

impl SpotifyClientWithToken {
    pub fn new(
        token: String,
        client: SpotifyClient,
    ) -> Self {
        Self {
            token: token,
            client: client,
        }
    }
}

pub struct TokenRing {
    current_token: SpotifyClientWithToken,
    ring: Arc<AtomicRingQueue<SpotifyClientWithToken>>,
    client: Arc<Client>,
}

impl TokenRing {
    pub fn init(
        client: Arc<Client>,
    ) -> Self {
        let mut tokens: Vec<SpotifyClientWithToken> = structs_from_file::<SpotifyClient>("clients.csv")
            .expect("Error in reading Spotify clients")
            .into_iter().map(|spotify_client| {
                SpotifyClientWithToken::new(
                    retrieve_access_token(
                        client.clone(),
                        &spotify_client.id[..],
                        &spotify_client.secret[..],
                    ).expect("Error in retrieving Spotify API tokens").access_token,
                    spotify_client,
                )
            }).collect();

        let token_ring = Self {
            // Extra slot to allow for pushing and popping in any order
            ring: Arc::new(AtomicRingQueue::with_capacity(tokens.len())),
            current_token: tokens.pop().expect("Empty Spotify clients"),
            client: client,
        };

        tokens.into_iter().map(|spotify_client_with_token| {
            token_ring.ring.push_overwrite(spotify_client_with_token);
        }).last();

        token_ring
    }

    pub fn front(
        &self,
    ) -> &String {
        &self.current_token.token
    }

    pub fn sleep_front_and_get_next(
        &mut self,
        secs: u64,
    ) {
        info!("Sleeping {} token {} seconds", self.current_token.client.name, secs);

        let ring_clone = self.ring.clone();
        let new_token = SpotifyClientWithToken::new(
            self.current_token.token.clone(),
            SpotifyClient {
                name: self.current_token.client.name.clone(),
                id: self.current_token.client.id.clone(),
                secret: self.current_token.client.secret.clone(),
            },
        );
        thread::spawn(move || {
            sleep(Duration::from_secs(secs));
            ring_clone.push_overwrite(new_token);
        });

        self.current_token = self.ring.pop();
        info!("Using {} token", self.current_token.client.name);
    }

    pub fn refresh_front_and_get_next(
        &mut self,
    ) {
        info!("Refreshing {} token", self.current_token.client.name);
        
        self.ring.push_overwrite(SpotifyClientWithToken::new(
            retrieve_access_token(
                self.client.clone(),
                &self.current_token.client.id[..],
                &self.current_token.client.secret[..],
            ).unwrap_or(AccessToken{
                access_token: self.current_token.token.clone(),
                token_type: String::new(),
                expires_in: 0,
                scope: String::new(),
            }).access_token,
            SpotifyClient {
                name: self.current_token.client.name.clone(),
                id: self.current_token.client.id.clone(),
                secret: self.current_token.client.secret.clone(),
            },
        ));
        
        self.current_token = self.ring.pop();
        info!("Using {} token", self.current_token.client.name);
    }
}

fn retrieve_access_token(
    client: Arc<Client>,
    id: &str,
    secret: &str,
) -> Result<AccessToken, reqwest::Error> {
    let mut form_data = HashMap::new();
    form_data.insert("grant_type", "client_credentials");

    Ok(
        client.post("https://accounts.spotify.com/api/token/")
            .basic_auth(id, Some(secret))
            .form(&form_data)
            .send()?
            .json()?
    )
}

