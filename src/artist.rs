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
        AlbumSimple,
    },
    artist_types::{
        ArtistFull,
    },
    client::{
        ClientRing,
    },
    common_types::{
        Paging,
    },
    track_types::{
        TrackFull,
    },
    utils::{
        get_with_retry,
        search,
        SimpleError,
    },
};

type CustomFuture<T> = Box<Future<Item = T, Error = Box<SimpleError>>>;

pub fn get_artist(
    client_ring: Arc<RwLock<ClientRing>>,
    artist_id: String,
) -> CustomFuture<ArtistFull> {
    Box::new(
        get_with_retry(
            format!("https://api.spotify.com/v1/artists/{}/", artist_id),
            client_ring,
        )
    )
}

pub fn get_artist_albums(
    client_ring: Arc<RwLock<ClientRing>>,
    artist_id: String,
) -> CustomFuture<Paging<AlbumSimple>> {
    Box::new(
        get_with_retry(
            format!("https://api.spotify.com/v1/artists/{}/albums/?include_groups=album,single,compilation&country=US", artist_id),
            client_ring,
        )
    )
}

pub fn get_artist_top_tracks(
    client_ring: Arc<RwLock<ClientRing>>,
    artist_id: String,
) -> CustomFuture<Vec<TrackFull>> {
    Box::new(
        get_with_retry::<serde_json::Value>(
            format!("https://api.spotify.com/v1/artists/{}/top-tracks/?country=US", artist_id),
            client_ring,
        ).map(|value| {
            value.get("tracks").expect("Error in artist::get_artist_top_tracks format")
                .as_array().expect("Error in artist::get_artist_top_tracks format")
                .into_iter().map(|value| {
                    serde_json::from_value::<TrackFull>(value.to_owned())
                        .expect("Error in artist::get_artist_top_tracks format")
                }).collect()
        })
    )
}

pub fn get_artist_related_artists(
    client_ring: Arc<RwLock<ClientRing>>,
    artist_id: String,
) -> CustomFuture<Vec<ArtistFull>> {
    Box::new(
        get_with_retry::<serde_json::Value>(
            format!("https://api.spotify.com/v1/artists/{}/related-artists/", artist_id),
            client_ring,
        ).map(|value| {
            value.get("artists").expect("Error in artist::get_artist_related_artists format")
                .as_array().expect("Error in artist::get_artist_related_artists format")
                .into_iter().map(|value| {
                    serde_json::from_value::<ArtistFull>(value.to_owned())
                        .expect("Error in artist::get_artist_related_artists format")
                }).collect()
        })
    )
}

pub fn get_artists(
    client_ring: Arc<RwLock<ClientRing>>,
    artist_ids: Vec<String>,
) -> CustomFuture<Vec<ArtistFull>> {
    Box::new(
        get_with_retry::<serde_json::Value>(
            format!(
                "https://api.spotify.com/v1/artists/?ids={}",
                artist_ids.join(",")
            ),
            client_ring,
        ).map(|value| {
            value.get("artists").expect("Error in artist::get_artists format")
                .as_array().expect("Error in artist::get_artists format")
                .iter().map(|value| {
                    serde_json::from_value::<ArtistFull>(value.to_owned())
                        .expect("Error in artist::get_artists format")
                }).collect()
        })
    )
}

pub fn search_artists(
    client_ring: Arc<RwLock<ClientRing>>,
    query: String,
) -> CustomFuture<Paging<ArtistFull>> {
    Box::new(
        search::<serde_json::Value>(query, "artist", client_ring)
            .map(|value| {
                serde_json::from_value(
                    value.get("artists").expect("Error in artist::search_artists format")
                        .to_owned()
                ).expect("Error in artist::search_artists format")
            })
    )
}
