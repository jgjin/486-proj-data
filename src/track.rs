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
    client::{
        ClientRing,
    },
    common_types::{
        Paging,
    },
    track_types::{
        AudioAnalysis,
        AudioFeatures,
        TrackFull,
    },
    utils::{
        get_with_retry,
        search,
        SimpleError,
    },
};

type CustomFuture<T> = Box<Future<Item = T, Error = Box<SimpleError>>>;

pub fn get_track_analysis(
    client_ring: Arc<RwLock<ClientRing>>,
    track_id: String,
) -> CustomFuture<AudioAnalysis> {
    Box::new(
        get_with_retry(
            format!("https://api.spotify.com/v1/audio-analysis/{}/", track_id),
            client_ring,
        )
    )
}

pub fn get_track_features(
    client_ring: Arc<RwLock<ClientRing>>,
    track_id: String,
) -> CustomFuture<AudioFeatures> {
    Box::new(
        get_with_retry(
            format!("https://api.spotify.com/v1/audio-features/{}/", track_id),
            client_ring,
        )
    )
}

pub fn get_tracks_features(
    client_ring: Arc<RwLock<ClientRing>>,
    track_ids: Vec<String>,
) -> CustomFuture<Vec<AudioFeatures>> {
    Box::new(
        get_with_retry::<serde_json::Value>(
            format!(
                "https://api.spotify.com/v1/audio-features/?ids={}",
                track_ids.join(","),
            ),
            client_ring,
        ).map(|value| {
            value.get("audio_features").expect("Error in tracks::get_tracks_features format")
                .as_array().expect("Error in track::get_tracks_features format")
                .iter().map(|value| {
                    serde_json::from_value::<AudioFeatures>(value.to_owned())
                        .expect("Error in track::get_tracks_features format")
                }).collect()
        })
    )
}

pub fn get_tracks(
    client_ring: Arc<RwLock<ClientRing>>,
    track_ids: Vec<String>,
) -> CustomFuture<Vec<TrackFull>> {
    Box::new(
        get_with_retry::<serde_json::Value>(
            format!(
                "https://api.spotify.com/v1/tracks/?ids={}",
                track_ids.join(","),
            ),
            client_ring,
        ).map(|value| {
            value.get("tracks").expect("Error in track::get_tracks format")
                .as_array().expect("Error in track::get_tracks format")
                .iter().map(|value| {
                    serde_json::from_value::<TrackFull>(value.to_owned())
                        .expect("Error in track::get_tracks format")
                }).collect()

        })
    )
}

pub fn get_track(
    client_ring: Arc<RwLock<ClientRing>>,
    track_id: String,
) -> CustomFuture<TrackFull> {
    Box::new(
        get_with_retry(
            format!("https://api.spotify.com/v1/tracks/{}/", track_id),
            client_ring,
        )
    )
}

pub fn search_tracks(
    client_ring: Arc<RwLock<ClientRing>>,
    query: String,
) -> CustomFuture<Paging<TrackFull>> {
    Box::new(
        search::<serde_json::Value>(query, "track", client_ring)
            .map(|value| {
                serde_json::from_value(
                    value.get("tracks").expect("Error in track::search_tracks format")
                        .to_owned()
                ).expect("Error in track::search_track format")
            })
    )
}
