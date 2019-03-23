use std::{
    sync::{
        Arc,
        RwLock,
    },
};

use futures::{
    Future,
};

use crate::{
    album_types::{
        AlbumFull,
        AlbumSimple,
    },
    client::{
        ClientRing,
    },
    common_types::{
        Paging,
    },
    track_types::{
        TrackSimple,
    },
    utils::{
        get_with_retry,
        search,
        SimpleError,
    },
};

type CustomFuture<T> = Box<Future<Item = T, Error = Box<SimpleError>>>;

pub fn get_album(
    client_ring: Arc<RwLock<ClientRing>>,
    album_id: String,
) -> CustomFuture<AlbumFull> {
    Box::new(
        get_with_retry(
            format!("https://api.spotify.com/v1/albums/{}/", album_id),
            client_ring,
        )
    )
}

pub fn get_album_tracks(
    client_ring: Arc<RwLock<ClientRing>>,
    album_id: String,
) -> CustomFuture<TrackSimple> {
    Box::new(
        get_with_retry(
            format!("https://api.spotify.com/v1/albums/{}/tracks/", album_id),
            client_ring,
        )
    )
}

pub fn get_albums(
    client_ring: Arc<RwLock<ClientRing>>,
    album_ids: Vec<String>,
) -> CustomFuture<Vec<AlbumFull>> {
    Box::new(
        get_with_retry::<serde_json::Value>(
            format!(
                "https://api.spotify.com/v1/albums/?ids={}",
                album_ids.join(","),
            ),
            client_ring,
        ).map(|value| {
            value.get("albums").expect("Error in album::get_albums format")
                .as_array().expect("Error in album::get_albums format")
                .into_iter().map(|value| {
                    serde_json::from_value::<AlbumFull>(value.to_owned())
                        .expect("Error in album::get_albums format")
                }).collect()
        })
    )
}

pub fn search_albums(
    client_ring: Arc<RwLock<ClientRing>>,
    query: String,
) -> CustomFuture<Paging<AlbumSimple>> {
    Box::new(
        search::<serde_json::Value>(query, "album", client_ring)
            .map(|value| {
                serde_json::from_value(
                    value.get("albums").expect("Error in album::search_albums format")
                        .to_owned()
                ).expect("Error in album::search_albums format")
            })
    )
}
