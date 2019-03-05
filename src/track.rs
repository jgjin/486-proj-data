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
    common_types::{
        Paging,
    },
    track_types::{
        AudioAnalysis,
        AudioFeatures,
        TrackFull,
    },
    utils::{
        search,
        get_with_retry,
    },
};

pub fn get_track_analysis(
    client: Arc<Client>,
    token: Arc<RwLock<String>>,
    track_id: &str,
) -> Result<AudioAnalysis, Box<dyn Error>> {
    Ok(
        get_with_retry(
            &format!("https://api.spotify.com/v1/audio-analysis/{}/", track_id)[..],
            client,
            token,
        )?.json()?
    )
}

pub fn get_track_features(
    client: Arc<Client>,
    token: Arc<RwLock<String>>,
    track_id: &str,
) -> Result<AudioFeatures, Box<dyn Error>> {
    Ok(
        get_with_retry(
            &format!("https://api.spotify.com/v1/audio-features/{}/", track_id)[..],
            client,
            token,
        )?.json()?
    )
}

pub fn get_tracks_features(
    client: Arc<Client>,
    token: Arc<RwLock<String>>,
    track_ids: Vec<&str>,
) -> Result<Vec<AudioFeatures>, Box<dyn Error>> {
    Ok(
        get_with_retry(
            &format!(
                "https://api.spotify.com/v1/audio-features/?ids={}",
                track_ids.join(","),
            )[..],
            client,
            token,
        )?.json::<serde_json::Value>()
            .expect("Error parsing JSON in tracks::get_tracks_features")
            .get("audio_features").expect("Error in tracks::get_tracks_features format")
            .as_array().expect("Error in track::get_tracks_features format")
            .iter().map(|value| {
                serde_json::from_value::<AudioFeatures>(value.to_owned())
                    .expect("Error in track::get_tracks_features format")
            }).collect()
    )
}

pub fn get_tracks(
    client: Arc<Client>,
    token: Arc<RwLock<String>>,
    track_ids: Vec<&str>,
) -> Result<Vec<TrackFull>, Box<dyn Error>> {
    Ok(
        get_with_retry(
            &format!(
                "https://api.spotify.com/v1/tracks/?ids={}",
                track_ids.join(","),
            )[..],
            client,
            token,
        )?.json::<serde_json::Value>()
            .expect("Error parsing JSON in track::get_tracks")
            .get("tracks").expect("Error in track::get_tracks format")
            .as_array().expect("Error in track::get_tracks format")
            .iter().map(|value| {
                serde_json::from_value::<TrackFull>(value.to_owned())
                    .expect("Error in track::get_tracks format")
            }).collect()
    )
}

pub fn get_track(
    client: Arc<Client>,
    token: Arc<RwLock<String>>,
    track_id: &str,
) -> Result<TrackFull, Box<dyn Error>> {
    Ok(
        get_with_retry(
            &format!("https://api.spotify.com/v1/tracks/{}/", track_id)[..],
            client,
            token,
        )?.json()?
    )
}

pub fn search_tracks(
    client: Arc<Client>,
    token: Arc<RwLock<String>>,
    query: &str,
) -> Result<Paging<TrackFull>, Box<dyn Error>> {
    Ok(
        serde_json::from_value(
            search(query, "track", client, token)?
                .json::<serde_json::Value>()
                .expect("Error parsing JSON in track::search_tracks")
                .get("tracks").expect("Error in track::search_tracks format")
                .to_owned()
        ).expect("Error in track::search_track format")
    )
}
