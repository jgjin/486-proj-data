use std::{
    sync::{
        Arc,
    },
};

use reqwest::{
    Client,
};

use crate::{
    album_types::{
        AlbumFull,
        AlbumSimple,
    },
    common_types::{
        Paging,
    },
    track_types::{
        TrackSimple,
    },
    utils::{
        search,
        get_with_retry,
    },
};

pub fn get_album(
    client: Arc<Client>,
    token: Arc<String>,
    album_id: &str,
) -> Result<AlbumFull, String> {
    Ok(
        get_with_retry(
            &format!("https://api.spotify.com/v1/albums/{}/", album_id)[..],
            client,
            token,
        )?.json().expect("Error parsing JSON in album::get_album")
    )
}

pub fn get_album_tracks(
    client: Arc<Client>,
    token: Arc<String>,
    album_id: &str,
) -> Result<Paging<TrackSimple>, String> {
    Ok(
        get_with_retry(
            &format!("https://api.spotify.com/v1/albums/{}/tracks/", album_id)[..],
            client,
            token,
        )?.json().expect("Error parsing JSON in album::get_album_tracks")
    )
}

pub fn get_albums(
    client: Arc<Client>,
    token: Arc<String>,
    album_ids: Vec<&str>,
) -> Result<Vec<AlbumFull>, String> {
    Ok(
        get_with_retry(
            &format!(
                "https://api.spotify.com/v1/albums/?ids={}",
                album_ids.join(","),
            )[..],
            client,
            token,
        )?.json::<serde_json::Value>()
            .expect("Error parsing JSON in album::get_albums")
            .get("albums").expect("Error in album::get_albums format")
            .as_array().expect("Error in album::get_albums format")
            .iter().map(|value| {
                serde_json::from_value::<AlbumFull>(value.to_owned())
                    .expect("Error in album::get_albums format")
            }).collect()
    )
}

pub fn search_albums(
    client: Arc<Client>,
    token: Arc<String>,
    query: &str,
) -> Result<Paging<AlbumSimple>, String> {
    Ok(
        serde_json::from_value(
            search(query, "album", client, token)?
                .json::<serde_json::Value>()
                .expect("Error parsing JSON in album::search_albums")
                .get("albums").expect("Error in album::search_albums format")
                .to_owned()
        ).expect("Error in album::search_albums format")
    )
}