use std::{
    borrow::{
        ToOwned,
    },
    cmp::{
        PartialOrd,
        Ordering,
    },
    ops::{
        Div,
        Mul,
        Sub,
    },
    iter::{
        Sum,
    },
};

use num::{
    One,
    pow::{
        pow,
    },
};
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

#[derive(Clone, Debug, Deserialize, Serialize)]
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
    pub track_id: String,
    pub origin_album: String,
    pub origin_album_name: String,
    pub origin_artist: String,
    pub origin_artist_name: String,
    pub origin_artist_genres: String,
    pub track_name: String,
    pub track_popularity: i32,
}

impl TrackCsv2 {
    pub fn extract_from(
        track_full: TrackFull,
        origin_artist: &ArtistCsv,
    ) -> Self {
        Self {
            track_id: track_full.id,
            origin_album: track_full.album.id,
            origin_album_name: track_full.album.name,
            origin_artist: origin_artist.id.clone(),
            origin_artist_name: origin_artist.name.clone(),
            origin_artist_genres: origin_artist.genres.clone(),
            track_name: track_full.name,
            track_popularity: track_full.popularity,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FeaturesCsv {
    pub track_id: String,
    // pub name: String,
    pub duration_ms: i32,
    pub key: i32,
    pub mode: i32,
    pub time_signature: i32,
    pub acousticness: f32,
    pub danceability: f32,
    pub energy: f32,
    pub instrumentalness: f32,
    pub liveness: f32,
    pub loudness: f32,
    pub speechiness: f32,
    pub valence: f32,
    pub tempo: f32,
    pub num_sections: usize,
    pub num_segments: usize,
    pub median_adj_pitch_0: f32,
    pub median_adj_pitch_1: f32,
    pub median_adj_pitch_2: f32,
    pub median_adj_pitch_3: f32,
    pub median_adj_pitch_4: f32,
    pub median_adj_pitch_5: f32,
    pub median_adj_pitch_6: f32,
    pub median_adj_pitch_7: f32,
    pub median_adj_pitch_8: f32,
    pub median_adj_pitch_9: f32,
    pub median_adj_pitch_10: f32,
    pub median_adj_pitch_11: f32,
    pub median_timbre_0: f32,
    pub median_timbre_1: f32,
    pub median_timbre_2: f32,
    pub median_timbre_3: f32,
    pub median_timbre_4: f32,
    pub median_timbre_5: f32,
    pub median_timbre_6: f32,
    pub median_timbre_7: f32,
    pub median_timbre_8: f32,
    pub median_timbre_9: f32,
    pub median_timbre_10: f32,
    pub median_timbre_11: f32,
    pub mean_adj_pitch_0: f32,
    pub mean_adj_pitch_1: f32,
    pub mean_adj_pitch_2: f32,
    pub mean_adj_pitch_3: f32,
    pub mean_adj_pitch_4: f32,
    pub mean_adj_pitch_5: f32,
    pub mean_adj_pitch_6: f32,
    pub mean_adj_pitch_7: f32,
    pub mean_adj_pitch_8: f32,
    pub mean_adj_pitch_9: f32,
    pub mean_adj_pitch_10: f32,
    pub mean_adj_pitch_11: f32,
    pub mean_timbre_0: f32,
    pub mean_timbre_1: f32,
    pub mean_timbre_2: f32,
    pub mean_timbre_3: f32,
    pub mean_timbre_4: f32,
    pub mean_timbre_5: f32,
    pub mean_timbre_6: f32,
    pub mean_timbre_7: f32,
    pub mean_timbre_8: f32,
    pub mean_timbre_9: f32,
    pub mean_timbre_10: f32,
    pub mean_timbre_11: f32,
    pub stdev_adj_pitch_0: f32,
    pub stdev_adj_pitch_1: f32,
    pub stdev_adj_pitch_2: f32,
    pub stdev_adj_pitch_3: f32,
    pub stdev_adj_pitch_4: f32,
    pub stdev_adj_pitch_5: f32,
    pub stdev_adj_pitch_6: f32,
    pub stdev_adj_pitch_7: f32,
    pub stdev_adj_pitch_8: f32,
    pub stdev_adj_pitch_9: f32,
    pub stdev_adj_pitch_10: f32,
    pub stdev_adj_pitch_11: f32,
    pub stdev_timbre_0: f32,
    pub stdev_timbre_1: f32,
    pub stdev_timbre_2: f32,
    pub stdev_timbre_3: f32,
    pub stdev_timbre_4: f32,
    pub stdev_timbre_5: f32,
    pub stdev_timbre_6: f32,
    pub stdev_timbre_7: f32,
    pub stdev_timbre_8: f32,
    pub stdev_timbre_9: f32,
    pub stdev_timbre_10: f32,
    pub stdev_timbre_11: f32,
    pub range_adj_pitch_0: f32,
    pub range_adj_pitch_1: f32,
    pub range_adj_pitch_2: f32,
    pub range_adj_pitch_3: f32,
    pub range_adj_pitch_4: f32,
    pub range_adj_pitch_5: f32,
    pub range_adj_pitch_6: f32,
    pub range_adj_pitch_7: f32,
    pub range_adj_pitch_8: f32,
    pub range_adj_pitch_9: f32,
    pub range_adj_pitch_10: f32,
    pub range_adj_pitch_11: f32,
    pub range_timbre_0: f32,
    pub range_timbre_1: f32,
    pub range_timbre_2: f32,
    pub range_timbre_3: f32,
    pub range_timbre_4: f32,
    pub range_timbre_5: f32,
    pub range_timbre_6: f32,
    pub range_timbre_7: f32,
    pub range_timbre_8: f32,
    pub range_timbre_9: f32,
    pub range_timbre_10: f32,
    pub range_timbre_11: f32,
}

impl Segment {
    pub fn get_key(
        &self,
        sections: &Vec<Section>,
    ) -> i32 {
        let segment_start = self.start;
        let segment_end = self.start + self.duration;
        sections.iter().map(|section| {
            let relevant_start = segment_start.max(section.start);
            let relevant_end = segment_end.min(section.start + section.duration);
            let prom = 0f32.max(relevant_end - relevant_start);
            (prom, Some(section))
        }).fold((0f32, None), |(max_prom_so_far, max_prom_section), (prom, next_section)| {
            max_prom_section.map(|section| {
                if max_prom_so_far.max(prom) == prom {
                    return (prom, next_section)
                }
                (max_prom_so_far, Some(section))
            }).unwrap_or((prom, next_section))
        }).1.map(|section_borrow| {
            section_borrow.key
        }).unwrap_or(0)
    }
}

#[derive(Default)]
struct DozenCollector<T: Default + PartialOrd + ToOwned> {
    pub values0: Vec<T>,
    pub values1: Vec<T>,
    pub values2: Vec<T>,
    pub values3: Vec<T>,
    pub values4: Vec<T>,
    pub values5: Vec<T>,
    pub values6: Vec<T>,
    pub values7: Vec<T>,
    pub values8: Vec<T>,
    pub values9: Vec<T>,
    pub values10: Vec<T>,
    pub values11: Vec<T>,
}

impl <T: Default + PartialOrd + ToOwned> DozenCollector<T> {
    pub fn from(
        collection_of_dozen: Vec<Vec<T>>,
    ) -> Self {
        let mut result: Self = Default::default();
        collection_of_dozen.into_iter().map(|dozen| {
            let mut iter = dozen.into_iter();
            result.values0.push(iter.next().unwrap_or(Default::default()));
            result.values1.push(iter.next().unwrap_or(Default::default()));
            result.values2.push(iter.next().unwrap_or(Default::default()));
            result.values3.push(iter.next().unwrap_or(Default::default()));
            result.values4.push(iter.next().unwrap_or(Default::default()));
            result.values5.push(iter.next().unwrap_or(Default::default()));
            result.values6.push(iter.next().unwrap_or(Default::default()));
            result.values7.push(iter.next().unwrap_or(Default::default()));
            result.values8.push(iter.next().unwrap_or(Default::default()));
            result.values9.push(iter.next().unwrap_or(Default::default()));
            result.values10.push(iter.next().unwrap_or(Default::default()));
            result.values11.push(iter.next().unwrap_or(Default::default()));
        }).last();

        let sort_t = |first: &T, second: &T| {
            first.partial_cmp(second).unwrap_or(Ordering::Equal)
        };
        result.values0.sort_unstable_by(sort_t);
        result.values1.sort_unstable_by(sort_t);
        result.values2.sort_unstable_by(sort_t);
        result.values3.sort_unstable_by(sort_t);
        result.values4.sort_unstable_by(sort_t);
        result.values5.sort_unstable_by(sort_t);
        result.values6.sort_unstable_by(sort_t);
        result.values7.sort_unstable_by(sort_t);
        result.values8.sort_unstable_by(sort_t);
        result.values9.sort_unstable_by(sort_t);
        result.values10.sort_unstable_by(sort_t);
        result.values11.sort_unstable_by(sort_t);
        
        result
    }
}

fn median_sorted_vec<T: Clone + Default>(
    vec: &Vec<T>,
) -> T {
    if vec.len() == 0 {
        return Default::default();
    }
    vec[vec.len() / 2].clone()
}

fn mean_sorted_vec<T: Clone + Default + Sum<T> + Div<f32, Output = T>>(
    vec: &Vec<T>,
) -> T {
    if vec.len() == 0 {
        return Default::default();
    }
    vec.iter().cloned().sum::<T>() / (vec.len() as f32)
}

fn stdev_sorted_vec<T: Clone + Default + Sum<T> + Div<f32, Output = T> + Mul<T> + Sub<T, Output = T> + One<Output = T>>(
    vec: &Vec<T>,
) -> T {
    if vec.len() == 0 {
        return Default::default();
    }
    let mean = mean_sorted_vec(vec);
    vec.iter().cloned().map(|elt| {
        pow(elt - mean.clone(), 2)
    }).sum::<T>() / (vec.len() as f32)
}

fn range_sorted_vec<T: Clone + Default + Sub<T, Output = T>>(
    vec: &Vec<T>,
) -> T {
    let max = vec.last().map(|val| val.clone()).unwrap_or(Default::default());
    let min = vec.first().map(|val| val.clone()).unwrap_or(Default::default());
    max - min
}

fn adjust_pitches(
    pitches: &mut Vec<f32>,
    key: i32,
) {
    pitches.rotate_left(key as usize);
}

impl FeaturesCsv {
    pub fn extract_from(
        mut analysis: AudioAnalysis,
        features: AudioFeatures,
    ) -> Self {
        let num_segments = analysis.segments.len();

        let sections = &analysis.sections;
        let adjusted_pitches = DozenCollector::from(
            analysis.segments.iter_mut().map(|segment| {
                let key = segment.get_key(sections);
                adjust_pitches(
                    &mut segment.pitches,
                    key,
                );
                segment.pitches.clone()
            }).collect()
        );

        let timbre = DozenCollector::from(
            analysis.segments.into_iter().map(|segment| {
                segment.timbre
            }).collect()
        );

        Self {
            track_id: features.id,
            duration_ms: features.duration_ms,
            key: features.key,
            mode: features.mode,
            time_signature: features.time_signature,
            acousticness: features.acousticness,
            danceability: features.danceability,
            energy: features.energy,
            instrumentalness: features.instrumentalness,
            liveness: features.liveness,
            loudness: features.loudness,
            speechiness: features.speechiness,
            valence: features.valence,
            tempo: features.tempo,
            num_sections: analysis.sections.len(),
            num_segments: num_segments,

            median_adj_pitch_0: median_sorted_vec(&adjusted_pitches.values0),
            median_adj_pitch_1: median_sorted_vec(&adjusted_pitches.values1),
            median_adj_pitch_2: median_sorted_vec(&adjusted_pitches.values2),
            median_adj_pitch_3: median_sorted_vec(&adjusted_pitches.values3),
            median_adj_pitch_4: median_sorted_vec(&adjusted_pitches.values4),
            median_adj_pitch_5: median_sorted_vec(&adjusted_pitches.values5),
            median_adj_pitch_6: median_sorted_vec(&adjusted_pitches.values6),
            median_adj_pitch_7: median_sorted_vec(&adjusted_pitches.values7),
            median_adj_pitch_8: median_sorted_vec(&adjusted_pitches.values8),
            median_adj_pitch_9: median_sorted_vec(&adjusted_pitches.values9),
            median_adj_pitch_10: median_sorted_vec(&adjusted_pitches.values10),
            median_adj_pitch_11: median_sorted_vec(&adjusted_pitches.values11),

            median_timbre_0: median_sorted_vec(&timbre.values0),
            median_timbre_1: median_sorted_vec(&timbre.values1),
            median_timbre_2: median_sorted_vec(&timbre.values2),
            median_timbre_3: median_sorted_vec(&timbre.values3),
            median_timbre_4: median_sorted_vec(&timbre.values4),
            median_timbre_5: median_sorted_vec(&timbre.values5),
            median_timbre_6: median_sorted_vec(&timbre.values6),
            median_timbre_7: median_sorted_vec(&timbre.values7),
            median_timbre_8: median_sorted_vec(&timbre.values8),
            median_timbre_9: median_sorted_vec(&timbre.values9),
            median_timbre_10: median_sorted_vec(&timbre.values10),
            median_timbre_11: median_sorted_vec(&timbre.values11),

            mean_adj_pitch_0: mean_sorted_vec(&adjusted_pitches.values0),
            mean_adj_pitch_1: mean_sorted_vec(&adjusted_pitches.values1),
            mean_adj_pitch_2: mean_sorted_vec(&adjusted_pitches.values2),
            mean_adj_pitch_3: mean_sorted_vec(&adjusted_pitches.values3),
            mean_adj_pitch_4: mean_sorted_vec(&adjusted_pitches.values4),
            mean_adj_pitch_5: mean_sorted_vec(&adjusted_pitches.values5),
            mean_adj_pitch_6: mean_sorted_vec(&adjusted_pitches.values6),
            mean_adj_pitch_7: mean_sorted_vec(&adjusted_pitches.values7),
            mean_adj_pitch_8: mean_sorted_vec(&adjusted_pitches.values8),
            mean_adj_pitch_9: mean_sorted_vec(&adjusted_pitches.values9),
            mean_adj_pitch_10: mean_sorted_vec(&adjusted_pitches.values10),
            mean_adj_pitch_11: mean_sorted_vec(&adjusted_pitches.values11),

            mean_timbre_0: mean_sorted_vec(&timbre.values0),
            mean_timbre_1: mean_sorted_vec(&timbre.values1),
            mean_timbre_2: mean_sorted_vec(&timbre.values2),
            mean_timbre_3: mean_sorted_vec(&timbre.values3),
            mean_timbre_4: mean_sorted_vec(&timbre.values4),
            mean_timbre_5: mean_sorted_vec(&timbre.values5),
            mean_timbre_6: mean_sorted_vec(&timbre.values6),
            mean_timbre_7: mean_sorted_vec(&timbre.values7),
            mean_timbre_8: mean_sorted_vec(&timbre.values8),
            mean_timbre_9: mean_sorted_vec(&timbre.values9),
            mean_timbre_10: mean_sorted_vec(&timbre.values10),
            mean_timbre_11: mean_sorted_vec(&timbre.values11),
            
            stdev_adj_pitch_0: stdev_sorted_vec(&adjusted_pitches.values0),
            stdev_adj_pitch_1: stdev_sorted_vec(&adjusted_pitches.values1),
            stdev_adj_pitch_2: stdev_sorted_vec(&adjusted_pitches.values2),
            stdev_adj_pitch_3: stdev_sorted_vec(&adjusted_pitches.values3),
            stdev_adj_pitch_4: stdev_sorted_vec(&adjusted_pitches.values4),
            stdev_adj_pitch_5: stdev_sorted_vec(&adjusted_pitches.values5),
            stdev_adj_pitch_6: stdev_sorted_vec(&adjusted_pitches.values6),
            stdev_adj_pitch_7: stdev_sorted_vec(&adjusted_pitches.values7),
            stdev_adj_pitch_8: stdev_sorted_vec(&adjusted_pitches.values8),
            stdev_adj_pitch_9: stdev_sorted_vec(&adjusted_pitches.values9),
            stdev_adj_pitch_10: stdev_sorted_vec(&adjusted_pitches.values10),
            stdev_adj_pitch_11: stdev_sorted_vec(&adjusted_pitches.values11),

            stdev_timbre_0: stdev_sorted_vec(&timbre.values0),
            stdev_timbre_1: stdev_sorted_vec(&timbre.values1),
            stdev_timbre_2: stdev_sorted_vec(&timbre.values2),
            stdev_timbre_3: stdev_sorted_vec(&timbre.values3),
            stdev_timbre_4: stdev_sorted_vec(&timbre.values4),
            stdev_timbre_5: stdev_sorted_vec(&timbre.values5),
            stdev_timbre_6: stdev_sorted_vec(&timbre.values6),
            stdev_timbre_7: stdev_sorted_vec(&timbre.values7),
            stdev_timbre_8: stdev_sorted_vec(&timbre.values8),
            stdev_timbre_9: stdev_sorted_vec(&timbre.values9),
            stdev_timbre_10: stdev_sorted_vec(&timbre.values10),
            stdev_timbre_11: stdev_sorted_vec(&timbre.values11),
            
            range_adj_pitch_0: range_sorted_vec(&adjusted_pitches.values0),
            range_adj_pitch_1: range_sorted_vec(&adjusted_pitches.values1),
            range_adj_pitch_2: range_sorted_vec(&adjusted_pitches.values2),
            range_adj_pitch_3: range_sorted_vec(&adjusted_pitches.values3),
            range_adj_pitch_4: range_sorted_vec(&adjusted_pitches.values4),
            range_adj_pitch_5: range_sorted_vec(&adjusted_pitches.values5),
            range_adj_pitch_6: range_sorted_vec(&adjusted_pitches.values6),
            range_adj_pitch_7: range_sorted_vec(&adjusted_pitches.values7),
            range_adj_pitch_8: range_sorted_vec(&adjusted_pitches.values8),
            range_adj_pitch_9: range_sorted_vec(&adjusted_pitches.values9),
            range_adj_pitch_10: range_sorted_vec(&adjusted_pitches.values10),
            range_adj_pitch_11: range_sorted_vec(&adjusted_pitches.values11),

            range_timbre_0: range_sorted_vec(&timbre.values0),
            range_timbre_1: range_sorted_vec(&timbre.values1),
            range_timbre_2: range_sorted_vec(&timbre.values2),
            range_timbre_3: range_sorted_vec(&timbre.values3),
            range_timbre_4: range_sorted_vec(&timbre.values4),
            range_timbre_5: range_sorted_vec(&timbre.values5),
            range_timbre_6: range_sorted_vec(&timbre.values6),
            range_timbre_7: range_sorted_vec(&timbre.values7),
            range_timbre_8: range_sorted_vec(&timbre.values8),
            range_timbre_9: range_sorted_vec(&timbre.values9),
            range_timbre_10: range_sorted_vec(&timbre.values10),
            range_timbre_11: range_sorted_vec(&timbre.values11),
        }
    }
}
