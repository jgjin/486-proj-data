use std::{
    sync::{
        Arc,
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
use serde_json::{
    Value,
};

pub fn search(
    query: &str,
    type_: &str,
    client: Arc<Client>,
    token: Arc<String>,
) -> Result<Response, String> {
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
    token: Arc<String>,
) -> Result<Response, String> {
    let response = client.get(url).bearer_auth(token.clone()).send().map_err(|err| {
        format!("Error for {}: {}", url, err)
    })?;
    match response.status() {
        StatusCode::OK => Ok(response),
        StatusCode::TOO_MANY_REQUESTS => {
            match response.headers().get(RETRY_AFTER) {
                Some(header_value) => {
                    let duration = header_value.to_str()
                        .expect("Unexpected format in retry-after header");
                    sleep(Duration::from_secs(
                        duration.parse::<u64>().expect("Unexpected format in retry-after header")
                    ));
                    get_with_retry(url, client, token)
                },
                None => Err("No retry-after header".to_string()),
            }
        },
        _ => Err("Unexpected error code".to_string()),
    }
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
        ()
    });
}
