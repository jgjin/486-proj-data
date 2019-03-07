use std::{
    error::{
        Error,
    },
    sync::{
        Arc,
        RwLock,
    },
};

use reqwest::{
    Client,
};

use crate::{
    album_types::{
        AlbumSimple,
    },
    artist_types::{
        ArtistFull,
    },
    common_types::{
        Paging,
    },
    token::{
        TokenRing,
    },
    track_types::{
        TrackFull,
    },
    utils::{
        search,
        get_with_retry,
    },
};

pub fn get_artist(
    client: Arc<Client>,
    token: Arc<RwLock<TokenRing>>,
    artist_id: &str,
) -> Result<ArtistFull, Box<dyn Error>> {
    Ok(
        get_with_retry(
            &format!("https://api.spotify.com/v1/artists/{}/", artist_id)[..],
            client,
            token,
        )?.json()?
    )
}

pub fn get_artist_albums(
    client: Arc<Client>,
    token: Arc<RwLock<TokenRing>>,
    artist_id: &str,
) -> Result<Paging<AlbumSimple>, Box<dyn Error>> {
    Ok(
        get_with_retry(
            &format!("https://api.spotify.com/v1/artists/{}/albums/?include_groups=album,single,compilation", artist_id)[..],
            client,
            token,
        )?.json()?
    )
}

pub fn get_artist_top_tracks(
    client: Arc<Client>,
    token: Arc<RwLock<TokenRing>>,
    artist_id: &str,
) -> Result<Vec<TrackFull>, Box<dyn Error>> {
    Ok(
        get_with_retry(
            &format!("https://api.spotify.com/v1/artists/{}/top-tracks/?country=US", artist_id)[..],
            client,
            token,
        )?.json::<serde_json::Value>()
            .expect("Error parsing JSON in artist::get_artist_top_tracks")
            .get("tracks").expect("Error in artist::get_artist_top_tracks format")
            .as_array().expect("Error in artist::get_artist_top_tracks format")
            .iter().map(|value| {
                serde_json::from_value::<TrackFull>(value.to_owned())
                    .expect("Error in artist::get_artist_top_tracks format")
            }).collect()
    )
}

pub fn get_artist_related_artists(
    client: Arc<Client>,
    token: Arc<RwLock<TokenRing>>,
    artist_id: &str,
) -> Result<Vec<ArtistFull>, Box<dyn Error>> {
    Ok(
        get_with_retry(
            &format!("https://api.spotify.com/v1/artists/{}/related-artists/", artist_id)[..],
            client,
            token,
        )?.json::<serde_json::Value>()
            .expect("Error parsing JSON in artist::get_artist_related_artists")
            .get("artists").expect("Error in artist::get_artist_related_artists format")
            .as_array().expect("Error in artist::get_artist_related_artists format")
            .iter().map(|value| {
                serde_json::from_value::<ArtistFull>(value.to_owned())
                    .expect("Error in artist::get_artist_related_artists format")
            }).collect()
    )
}

pub fn get_artists(
    client: Arc<Client>,
    token: Arc<RwLock<TokenRing>>,
    artist_ids: Vec<&str>,
) -> Result<Vec<ArtistFull>, Box<dyn Error>> {
    Ok(
        get_with_retry(
            &format!(
                "https://api.spotify.com/v1/artists/?ids={}",
                artist_ids.join(",")
            )[..],
            client,
            token,
        )?.json::<serde_json::Value>()
            .expect("Error parsing JSON in artist::get_artists")
            .get("artists").expect("Error in artist::get_artists format")
            .as_array().expect("Error in artist::get_artists format")
            .iter().map(|value| {
                serde_json::from_value::<ArtistFull>(value.to_owned())
                    .expect("Error in artist::get_artists format")
            }).collect()
    )
}

pub fn search_artists(
    client: Arc<Client>,
    token: Arc<RwLock<TokenRing>>,
    query: &str,
) -> Result<Paging<ArtistFull>, Box<dyn Error>> {
    Ok(
        serde_json::from_value(
            search(query, "artist", client, token)?
                .json::<serde_json::Value>()
                .expect("Error parsing JSON in artist::search_artists")
                .get("artists").expect("Error in artist::search_artists format")
                .to_owned()
        ).expect("Error in artist::search_artists format")
    )
}
