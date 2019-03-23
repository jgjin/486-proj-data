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

use futures::{
    future,
    Future,
};
use reqwest::{
    StatusCode,
    r#async::{
        Response,
    },
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
use tokio::{
    runtime::{
        current_thread::{
            Runtime,
        },
    },
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

type CustomFuture<T> = Box<Future<Item = T, Error = Box<SimpleError>>>;

pub fn search<D: 'static + DeserializeOwned>(
    query: String,
    type_: &str,
    client_ring: Arc<RwLock<ClientRing>>,
) -> CustomFuture<D> {
    get_with_retry::<D>(
        format!(
            "https://api.spotify.com/v1/search/?q={}&type={}",
            query.replace(" ", "%20"),
            type_,
        ),
        client_ring,
    )
}

pub fn get_with_retry<D: 'static + DeserializeOwned>(
    url: String,
    client_ring: Arc<RwLock<ClientRing>>,
) -> CustomFuture<D> {
    debug!("Getting URL {}", url);
    let (client, token) = client_ring.read().expect("client ring RwLock poisoned").front();
    Box::new(
        client.get(&url[..])
            .header(reqwest::header::AUTHORIZATION, &*format!("Bearer {}", token))
            .send().map_err(|err| SimpleError {
                message: err.to_string(),
            }.into()).and_then(|mut response| {
                match response.status() {
                    StatusCode::OK => Box::new(response.json::<D>().map_err(|err| SimpleError {
                        message: err.to_string(),
                    }.into())),
                    StatusCode::TOO_MANY_REQUESTS => {
                        match response.headers().get(RETRY_AFTER) {
                            Some(header_value) => {
                                let duration = header_value.to_str()
                                    .expect("Unexpected format in retry-after header");
                                (*client_ring.write().expect("client ring RwLock poisoned")).sleep_front_and_get_next(
                                    duration.parse::<u64>().expect("Unexpected format in retry-after header")
                                );
                                get_with_retry::<D>(url, client_ring)
                            },
                            None => Box::new(future::err(Box::new(SimpleError {
                                message: "No retry-after header".to_string(),
                            }))),
                        }
                    },
                    StatusCode::UNAUTHORIZED => {
                        (*client_ring.write().expect("client ring RwLock poisoned")).refresh_front_and_get_next();
                        get_with_retry::<D>(url, client_ring)
                    },
                    status_code => {
                        Box::new(future::err(Box::new(SimpleError {
                            message: format!("Unexpected error code: {}", status_code),
                        })))
                    },
                }
            })
    )
}

pub fn get_next_paging<D: 'static + DeserializeOwned>(
    client_ring: Arc<RwLock<ClientRing>>,
    url: String,
) -> CustomFuture<Paging<D>> {
    Box::new(
        get_with_retry(
            url,
            client_ring,
        )
    )
}

pub fn loop_until_ok<Input: Clone, OkReturn>(
    api_endpoint: &'static Fn(
        Arc<RwLock<ClientRing>>,
        Input,
    ) -> CustomFuture<OkReturn>, 
    client_ring: Arc<RwLock<ClientRing>>,
    input: Input,
) -> CustomFuture<OkReturn> {
    Box::new(
        api_endpoint(
            client_ring.clone(),
            input.clone(),
        ).or_else(move |_| {
            info!("Error in utils::loop_until_ok, retrying");
            thread::sleep(Duration::from_secs(3));
            loop_until_ok(
                api_endpoint,
                client_ring,
                input,
            )
        })
    )
}


#[allow(dead_code)]
pub fn print_full_response(
    response: &mut Response,
) {
    let mut rt = Runtime::new().expect("No tokio runtime");

    println!("url: {}", response.url().as_str());
    println!("status: {}", response.status());
    response.headers().iter().map(|header| {
        println!(
            "header {}: {}",
            header.0.as_str(),
            header.1.to_str().unwrap_or("<contains non-ASCII chars>"),
        );
    }).last();
    rt.block_on(response.json().map(|json: Value| {
        println!("{:?}", json);
    })).unwrap_or_else(|_| {
        println!("response not json");
    });
}
