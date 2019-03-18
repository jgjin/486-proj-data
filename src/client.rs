use std::{
    clone::{
        Clone,
    },
    collections::{
        HashMap,
    },
    error::{
        Error,
    },
    net::{
        Ipv4Addr,
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
    self,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
struct SpotifyClientMetadata {
    pub name: String,
    pub id: String,
    pub secret: String,
}

#[derive(Clone, Debug)]
struct SpotifyClientWithProxy {
    pub client_metadata: SpotifyClientMetadata,
    pub client: Client,
    pub proxy: Proxy,
    pub token: String,
}

impl SpotifyClientWithProxy {
    pub fn init(
        client_metadata: SpotifyClientMetadata,
        proxy: Proxy,
    ) -> reqwest::Result<Self> {
        let client = Client::builder()
            .proxy(reqwest::Proxy::all(&format!("{}:{}", proxy.ip_address, proxy.port)[..])?)
            .build()?;

        let token = retrieve_access_token(
            &client,
            &client_metadata.id[..],
            &client_metadata.secret[..],
        )?.access_token;
        
        Ok(Self {
            client_metadata: client_metadata,
            client: client,
            proxy: proxy,
            token: token,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Proxy {
    pub ip_address: Ipv4Addr,
    pub port: u16,
}

pub struct ClientRing {
    current_client: SpotifyClientWithProxy,
    client_ring: Arc<AtomicRingQueue<SpotifyClientWithProxy>>,
    proxies: Arc<AtomicRingQueue<Proxy>>,
}

impl ClientRing {
    pub fn init(
    ) -> Result<Self, Box<dyn Error>> {
        let clients_metadata = structs_from_file::<SpotifyClientMetadata>("clients.csv")?;
        let proxies = structs_from_file::<Proxy>("proxies.csv")?;

        let client_metadata_len = clients_metadata.len();
        let mut clients_with_proxies = clients_metadata.into_iter().zip(
            proxies.iter().cloned().cycle().take(client_metadata_len),
        ).map(|(client_metadata, proxy)| {
            SpotifyClientWithProxy::init(client_metadata, proxy)
        }).collect::<reqwest::Result<Vec<SpotifyClientWithProxy>>>()?;

        let current_client = clients_with_proxies.pop().expect("Empty clients or proxies");
        let client_ring = Arc::new(AtomicRingQueue::with_capacity(client_metadata_len * 2));
        clients_with_proxies.into_iter().map(|client_with_proxy| {
            client_ring.push_overwrite(client_with_proxy);
        }).last();
        
        let proxies_queue = Arc::new(AtomicRingQueue::with_capacity(
            (client_metadata_len + proxies.len()) * 2,
        ));
        proxies.into_iter().map(|proxy| { proxies_queue.push_overwrite(proxy); }).last();

        Ok(Self {
            current_client: current_client,
            client_ring: client_ring,
            proxies: proxies_queue,
        })
    }

    pub fn front(
        &self,
    ) -> (Client, String) {
        (self.current_client.client.clone(), self.current_client.token.clone())
    }

    pub fn sleep_front_and_get_next(
        &mut self,
        secs: u64,
    ) {
        info!("Sleeping {} client {} seconds", self.current_client.client_metadata.name, secs);

        let ring_clone = self.client_ring.clone();
        let current_client_clone = self.current_client.clone();
        thread::spawn(move || {
            sleep(Duration::from_secs(secs));
            ring_clone.try_push(current_client_clone)
                .expect("Error in pushing new client in sleep");
        });

        self.current_client = self.client_ring.pop();
        info!("Using {} client", self.current_client.client_metadata.name);
    }

    pub fn refresh_front_and_get_next(
        &mut self,
    ) {
        info!("Refreshing {} client", self.current_client.client_metadata.name);

        self.proxies.try_push(self.current_client.proxy.clone())
            .expect("Error in pushing new proxy in refresh");

        self.client_ring.try_push(SpotifyClientWithProxy::init(
            self.current_client.client_metadata.clone(),
            self.proxies.pop(),
        ).expect("Error in refreshing client")).expect("Error in pushing new client in refresh");

        self.current_client = self.client_ring.pop();
        info!("Using {} client", self.current_client.client_metadata.name);
    }
}

fn retrieve_access_token(
    client: &Client,
    id: &str,
    secret: &str,
) -> reqwest::Result<AccessToken> {
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
