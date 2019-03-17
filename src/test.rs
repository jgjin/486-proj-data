use std::{
    sync::{
        Arc,
        RwLock,
    },
};

use reqwest::{
    Client,
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
    let album_data = album::get_album(
        client.clone(),
        client_ring.clone(),
        "0sNOF9WDwhWunNAHPD3Baj"
    ).expect("Error in album::get_album");

    let album_tracks = album::get_album_tracks(
        client.clone(),
        client_ring.clone(),
        "6akEvsycLGftJxYudPjmqK",
    ).expect("Error in album::get_album_tracks");

    let albums = album::get_albums(
        client.clone(),
        client_ring.clone(),
        vec![
            "41MnTivkwTO3UUJ8DrqEJJ",
            "6JWc4iAiJ9FjyK0B59ABb4",
            "6UXCm6bOO4gFlDQZV5yL37",
        ],
    ).expect("Error in album::get_albums");

    let artist_data = artist::get_artist(
        client.clone(),
        client_ring.clone(),
        "0OdUWJ0sBjDrqHygGUXeCF",
    ).expect("Error in artist::get_artist");

    let artist_albums = artist::get_artist_albums(
        client.clone(),
        client_ring.clone(),
        "0OdUWJ0sBjDrqHygGUXeCF",
    ).expect("Error in artist::get_artist_albums");

    let artist_top_tracks = artist::get_artist_top_tracks(
        client.clone(),
        client_ring.clone(),
        "43ZHCT0cAZBISjO8DG9PnE",
    ).expect("Error in artist::get_artist_top_tracks");

    let artist_related_artists = artist::get_artist_related_artists(
        client.clone(),
        client_ring.clone(),
        "43ZHCT0cAZBISjO8DG9PnE",
    ).expect("Error in artist::get_artist_related_artists");

    let artists = artist::get_artists(
        client.clone(),
        client_ring.clone(),
        vec![
            "0oSGxfWSnnOXhD2fKuz2Gy",
            "3dBVyJ7JuOMt4GE9607Qin",
        ],
    ).expect("Error in artist::get_artists");

    let track_analysis = track::get_track_analysis(
        client.clone(),
        client_ring.clone(),
        "3JIxjvbbDrA9ztYlNcp3yL",
    ).expect("Error in track::get_track_analysis");

    let track_features = track::get_track_features(
        client.clone(),
        client_ring.clone(),
        "06AKEBrKUckW0KREUWRnvT",
    ).expect("Error in track::get_track_features");

    let tracks_features = track::get_tracks_features(
        client.clone(),
        client_ring.clone(),
        vec![
            "4JpKVNYnVcJ8tuMKjAj50A",
            "2NRANZE9UCmPAS5XVbXL40",
            "24JygzOLM0EmRQeGtFcIcG",
        ],
    ).expect("Error in track::get_tracks_features");

    let tracks = track::get_tracks(
        client.clone(),
        client_ring.clone(),
        vec![
            "11dFghVXANMlKmJXsNCbNl",
            "20I6sIOMTCkB6w7ryavxtO",
            "7xGfFoTpQ2E7fRF5lN10tr",
        ],
    ).expect("Error in track::get_tracks");

    let track_data = track::get_track(
        client.clone(),
        client_ring.clone(),
        "11dFghVXANMlKmJXsNCbNl",
    ).expect("Error in track::get_track");

    println!("Endpoint tests passed!");
}

#[allow(dead_code)]
pub fn test_searches(
    client: Arc<Client>,
    client_ring: Arc<RwLock<ClientRing>>,
) {
    let album_results = album::search_albums(client.clone(), client_ring.clone(), "Twin Fantasy")
        .expect("Error in album::search_albums");

    println!("Twin Fantasy albums:\n{:#?}", album_results.items[0]);

    let artist_results = artist::search_artists(client.clone(), client_ring.clone(), "Brockhampton")
        .expect("Error in artist::search_artists");

    println!("Brockhampton artists:\n{:#?}", artist_results.items[0]);

    let track_results = track::search_tracks(client.clone(), client_ring.clone(), "Shirim")
        .expect("Error in track::search_tracks");

    println!("Shirim tracks:\n{:#?}", track_results.items[0]);
}
