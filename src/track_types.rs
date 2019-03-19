use serde::{
    Deserialize,
    Serialize,
};
use serde_json::{
    Map,
    Value,
};

use crate::{
    album_types::{
        AlbumSimple,
    },
    artist_types::{
        ArtistCsv,
        ArtistSimple,
    },
};

#[derive(Debug, Deserialize, Serialize)]
pub struct TimeInterval {
    pub start: f32,
    pub duration: f32,
    pub confidence: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Section {
    pub start: f32,
    pub duration: f32,
    pub confidence: f32,
    pub loudness: f32,
    pub tempo: f32,
    pub tempo_confidence: f32,
    pub key: i32,
    pub key_confidence: f32,
    pub mode: i32,
    pub mode_confidence: f32,
    pub time_signature: i32,
    pub time_signature_confidence: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Segment {
    pub start: f32,
    pub duration: f32,
    pub confidence: f32,
    pub loudness_start: f32,
    pub loudness_max: f32,
    pub loudness_max_time: f32,
    pub loudness_end: Option<f32>,
    pub pitches: Vec<f32>,
    pub timbre: Vec<f32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AudioAnalysis {
    pub bars: Vec<TimeInterval>,
    pub beats: Vec<TimeInterval>,
    pub sections: Vec<Section>,
    pub segments: Vec<Segment>,
    pub tatums: Vec<TimeInterval>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AudioFeatures {
    pub acousticness: f32,
    pub analysis_url: String,
    pub danceability: f32,
    pub duration_ms: i32,
    pub energy: f32,
    pub id: String,
    pub instrumentalness: f32,
    pub key: i32,
    pub liveness: f32,
    pub loudness: f32,
    pub mode: i32,
    pub speechiness: f32,
    pub tempo: f32,
    pub time_signature: i32,
    pub track_href: String,
    pub uri: String,
    pub valence: f32,
    #[serde(rename = "type")] 
    pub object_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackLink {
    external_urls: Map<String, Value>,
    href: String,
    id: String,
    uri: String,
    #[serde(rename = "type")]
    pub object_type: String,
}

macro_rules! with_track_core_fields {
    (pub struct $name:ident { $( pub $field:ident: $ty:ty ),* $(,)* }) => {
        #[derive(Debug, Deserialize, Serialize)]
        pub struct $name {
            pub artists: Vec<ArtistSimple>,
            pub available_markets: Option<Vec<String>>,
            pub disc_number: i32,
            pub duration_ms: i32,
            pub explicit: bool,
            pub external_urls: Map<String, Value>,
            pub href: String,
            pub id: String,
            pub is_playable: Option<bool>,
            pub linked_from: Option<TrackLink>,
            pub name: String,
            pub preview_url: Option<String>,
            pub track_number: i32,
            pub uri: String,
            #[serde(rename = "type")]
            pub object_type: String,
            $( pub $field: $ty ),*
        }
    };
}

with_track_core_fields!(pub struct TrackSimple {});

with_track_core_fields!(pub struct TrackFull {
    pub album: AlbumSimple,
    pub external_ids: Map<String, Value>,
    pub popularity: i32,
    pub restrictions: Option<Map<String, Value>>,
});

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackCsv {
    pub origin_album: String,
    pub origin_album_or_origin_artist_genres: String,
    pub id: String,
    pub name: String,
    pub track_number: i32,
}

impl TrackCsv {
    pub fn extract_from(
        track_simple: TrackSimple,
        origin_album: String,
        origin_album_or_origin_artist_genres: String,
    ) -> Self {
        Self {
            origin_album: origin_album,
            origin_album_or_origin_artist_genres: origin_album_or_origin_artist_genres,
            id: track_simple.id,
            name: track_simple.name,
            track_number: track_simple.track_number,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackCsv2 {
    pub origin_album: String,
    pub origin_album_name: String,
    pub origin_artist: String,
    pub origin_artist_name: String,
    pub origin_artist_genres: String,
    pub track_id: String,
    pub track_name: String,
    pub track_popularity: i32,
}

impl TrackCsv2 {
    pub fn extract_from(
        track_full: TrackFull,
        origin_artist: &ArtistCsv,
    ) -> Self {
        Self {
            origin_album: track_full.album.id,
            origin_album_name: track_full.album.name,
            origin_artist: origin_artist.id.clone(),
            origin_artist_name: origin_artist.name.clone(),
            origin_artist_genres: origin_artist.genres.clone(),
            track_id: track_full.id,
            track_name: track_full.name,
            track_popularity: track_full.popularity,
        }
    }
}
