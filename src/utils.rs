use std::{
    clone::{
        Clone,
    },
    fmt::{
        Display,
        Formatter,
        self,
    },
    error::{
        Error,
    },
    sync::{
        Arc,
        RwLock,
    },
    thread,
    time::{
        Duration,
    },
};

use reqwest::{
    Client,
    Response,
    StatusCode,
    header::{
        RETRY_AFTER,
    },
};
use serde::{
    de::{
        DeserializeOwned,
    },
};
use serde_json::{
    Value,
};

use crate::{
    client::{
        ClientRing,
    },
    common_types::{
        Paging,
    },
};

#[derive(Debug, Clone)]
pub struct SimpleError {
    pub message: String,
}

impl Display for SimpleError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl Error for SimpleError {
    fn description(&self) -> &str {
        &self.message[..]
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

pub fn search(
    query: &str,
    type_: &str,
    client: Arc<Client>,
    client_ring: Arc<RwLock<ClientRing>>,
) -> Result<Response, Box<dyn Error>> {
    get_with_retry(
        &format!(
            "https://api.spotify.com/v1/search/?q={}&type={}",
            query.replace(" ", "%20"),
            type_,
        )[..],
        client,
        client_ring,
    )
}

pub fn get_with_retry(
    url: &str,
    client: Arc<Client>,
    client_ring: Arc<RwLock<ClientRing>>,
) -> Result<Response, Box<dyn Error>> {
    debug!("Getting URL {}", url);
    let response = client.get(url)
        .bearer_auth(client_ring.read().expect("client ring RwLock poisoned").front())
        .send().map_err(|err| {
            format!("Error for {}: {}", url, err)
        })?;
    match response.status() {
        StatusCode::OK => Ok(response),
        StatusCode::TOO_MANY_REQUESTS => {
            match response.headers().get(RETRY_AFTER) {
                Some(header_value) => {
                    let duration = header_value.to_str()
                        .expect("Unexpected format in retry-after header");
                    (*client_ring.write().expect("client ring RwLock poisoned")).sleep_front_and_get_next(
                        duration.parse::<u64>().expect("Unexpected format in retry-after header")
                    );
                    get_with_retry(url, client, client_ring)
                },
                None => Err(Box::new(SimpleError {
                    message: "No retry-after header".to_string(),
                })),
            }
        },
        StatusCode::UNAUTHORIZED => {
            (*client_ring.write().expect("client ring RwLock poisoned")).refresh_front_and_get_next();
            get_with_retry(url, client, client_ring)
        },
        status_code => Err(Box::new(SimpleError {
            message: format!("Unexpected error code: {}", status_code)
        })),
    }
}

pub fn get_next_paging<D: DeserializeOwned>(
    client: Arc<Client>,
    client_ring: Arc<RwLock<ClientRing>>,
    url: &str,
) -> Result<Paging<D>, Box<dyn Error>> {
    Ok(
        get_with_retry(
            url,
            client,
            client_ring,
        )?.json()?
    )
}

pub fn loop_until_ok<Input: Clone, OkReturn>(
    api_endpoint: &Fn(
        Arc<Client>,
        Arc<RwLock<ClientRing>>,
        Input,
    ) -> Result<OkReturn, Box<dyn Error>>, 
    client: Arc<Client>,
    client_ring: Arc<RwLock<ClientRing>>,
    input: Input,
) -> Result<OkReturn, Box<dyn Error>> {
    api_endpoint(
        client.clone(),
        client_ring.clone(),
        input.clone(),
    ).or_else(|_| {
        info!("Error in utils::loop_until_ok, retrying");
        thread::sleep(Duration::from_secs(3));
        loop_until_ok(
            api_endpoint,
            client,
            client_ring,
            input,
        )
    })
}


#[allow(dead_code)]
pub fn print_full_response(
    response: &mut Response,
) {
    println!("url: {}", response.url().as_str());
    println!("status: {}", response.status());
    response.headers().iter().map(|header| {
        println!(
            "header {}: {}",
            header.0.as_str(),
            header.1.to_str().unwrap_or("<contains non-ASCII chars>"),
        );
    }).last();
    response.json().map(|json: Value| {
        println!("{:?}", json);
    }).unwrap_or_else(|_| {
        println!("response not json");
    });
}
