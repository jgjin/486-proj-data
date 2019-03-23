use std::{
    sync::{
        Arc,
        RwLock,
    },
};

use reqwest::{
    r#async::{
        Client,
    },
};
use tokio::{
    runtime::{
        current_thread::{
            Runtime,
        },
    },
};

use crate::{
    album,
    artist,
    client::{
        ClientRing,
    },
    track,
};

#[allow(dead_code, unused_variables)]
pub fn test_endpoints(
    client: Arc<Client>,
    client_ring: Arc<RwLock<ClientRing>>,
) {
    let mut rt = Runtime::new().expect("No tokio runtime");
    
    let album_data = rt.block_on(album::get_album(
        client_ring.clone(),
        "0sNOF9WDwhWunNAHPD3Baj".to_string(),
    )).expect("Error in album::get_album");

    let album_tracks = rt.block_on(album::get_album_tracks(
        client_ring.clone(),
        "6akEvsycLGftJxYudPjmqK".to_string(),
    )).expect("Error in album::get_album_tracks");

    let albums = rt.block_on(album::get_albums(
        client_ring.clone(),
        vec![
            "41MnTivkwTO3UUJ8DrqEJJ".to_string(),
            "6JWc4iAiJ9FjyK0B59ABb4".to_string(),
            "6UXCm6bOO4gFlDQZV5yL37".to_string(),
        ],
    )).expect("Error in album::get_albums");

    let artist_data = rt.block_on(artist::get_artist(
        client_ring.clone(),
        "0OdUWJ0sBjDrqHygGUXeCF".to_string(),
    )).expect("Error in artist::get_artist");

    let artist_albums = rt.block_on(artist::get_artist_albums(
        client_ring.clone(),
        "0OdUWJ0sBjDrqHygGUXeCF".to_string(),
    )).expect("Error in artist::get_artist_albums");

    let artist_top_tracks = rt.block_on(artist::get_artist_top_tracks(
        client_ring.clone(),
        "43ZHCT0cAZBISjO8DG9PnE".to_string(),
    )).expect("Error in artist::get_artist_top_tracks");

    let artist_related_artists = rt.block_on(artist::get_artist_related_artists(
        client_ring.clone(),
        "43ZHCT0cAZBISjO8DG9PnE".to_string(),
    )).expect("Error in artist::get_artist_related_artists");

    let artists = rt.block_on(artist::get_artists(
        client_ring.clone(),
        vec![
            "0oSGxfWSnnOXhD2fKuz2Gy".to_string(),
            "3dBVyJ7JuOMt4GE9607Qin".to_string(),
        ],
    )).expect("Error in artist::get_artists");

    let track_analysis = rt.block_on(track::get_track_analysis(
        client_ring.clone(),
        "3JIxjvbbDrA9ztYlNcp3yL".to_string(),
    )).expect("Error in track::get_track_analysis");

    let track_features = rt.block_on(track::get_track_features(
        client_ring.clone(),
        "06AKEBrKUckW0KREUWRnvT".to_string(),
    )).expect("Error in track::get_track_features");

    let tracks_features = rt.block_on(track::get_tracks_features(
        client_ring.clone(),
        vec![
            "4JpKVNYnVcJ8tuMKjAj50A".to_string(),
            "2NRANZE9UCmPAS5XVbXL40".to_string(),
            "24JygzOLM0EmRQeGtFcIcG".to_string(),
        ],
    )).expect("Error in track::get_tracks_features");

    let tracks = rt.block_on(track::get_tracks(
        client_ring.clone(),
        vec![
            "11dFghVXANMlKmJXsNCbNl".to_string(),
            "20I6sIOMTCkB6w7ryavxtO".to_string(),
            "7xGfFoTpQ2E7fRF5lN10tr".to_string(),
        ],
    )).expect("Error in track::get_tracks");

    let track_data = rt.block_on(track::get_track(
        client_ring.clone(),
        "11dFghVXANMlKmJXsNCbNl".to_string(),
    )).expect("Error in track::get_track");

    println!("Endpoint tests passed!");
}

#[allow(dead_code)]
pub fn test_searches(
    client_ring: Arc<RwLock<ClientRing>>,
) {
    let mut rt = Runtime::new().expect("No tokio runtime");
    
    let album_results = rt.block_on(
        album::search_albums(client_ring.clone(), "Twin Fantasy".to_string())
    ).expect("Error in album::search_albums");

    println!("Twin Fantasy albums:\n{:#?}", album_results.items[0]);

    let artist_results = rt.block_on(
        artist::search_artists(client_ring.clone(), "Brockhampton".to_string())
    ).expect("Error in artist::search_artists");

    println!("Brockhampton artists:\n{:#?}", artist_results.items[0]);

    let track_results = rt.block_on(
        track::search_tracks(client_ring.clone(), "Shirim".to_string())
    ).expect("Error in track::search_tracks");

    println!("Shirim tracks:\n{:#?}", track_results.items[0]);
}
