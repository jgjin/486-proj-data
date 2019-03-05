use std::{
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
    thread::{
        sleep,
    },
    time::{
        Duration,
    }
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
    token::{
        retrieve_access_token,
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
    token: Arc<RwLock<String>>,
) -> Result<Response, Box<dyn Error>> {
    get_with_retry(
        &format!(
            "https://api.spotify.com/v1/search/?q={}&type={}",
            query.replace(" ", "%20"),
            type_,
        )[..],
        client,
        token,
    )
}

pub fn get_with_retry(
    url: &str,
    client: Arc<Client>,
    token: Arc<RwLock<String>>,
) -> Result<Response, Box<dyn Error>> {
    info!("Getting URL {}", url);
    let response = client.get(url)
        .bearer_auth(token.read().expect("token RwLock poisoned"))
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
                    info!("Sleeping {} seconds", duration);
                    sleep(Duration::from_secs(
                        duration.parse::<u64>().expect("Unexpected format in retry-after header")
                    ));

                    get_with_retry(url, client, token)
                },
                None => Err(Box::new(SimpleError {
                    message: "No retry-after header".to_string(),
                })),
            }
        },
        StatusCode::UNAUTHORIZED => {
            info!("Refreshing token");
            *(token.write().expect("token RwLock poisoned")) = retrieve_access_token(client.clone())
                .expect("Error in access token")
                .access_token;
            info!("Using token {}", token.read().expect("token RwLock poisoned"));
                
            get_with_retry(url, client, token)
        },
        status_code => Err(Box::new(SimpleError {
            message: format!("Unexpected error code: {}", status_code)
        })),
    }
}

pub fn get_next_paging<D: DeserializeOwned>(
    url: &str,
    client: Arc<Client>,
    token: Arc<RwLock<String>>,
) -> Result<Paging<D>, Box<dyn Error>> {
    Ok(
        get_with_retry(
            url,
            client,
            token,
        )?.json()?
    )
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
