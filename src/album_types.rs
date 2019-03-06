use serde::{
    Deserialize,
    Serialize,
};
use serde_json::{
    Map,
    Value,
};

use crate::{
    artist_types::{
        ArtistSimple,
    },
    common_types::{
        Image,
        Paging,
    },
    track_types::{
        TrackSimple,
    },
};

macro_rules! with_album_core_fields {
    (pub struct $name:ident { $( pub $field:ident: $ty:ty ),* $(,)* }) => {
        #[derive(Debug, Deserialize, Serialize)]
        pub struct $name {
            pub album_group: Option<String>,
            pub album_type: String,
            pub artists: Vec<ArtistSimple>,
            pub available_markets: Option<Vec<String>>,
            pub external_urls: Map<String, Value>,
            pub href: String,
            pub id: String,
            pub images: Vec<Image>,
            pub name: String,
            pub release_date: String,
            pub release_date_precision: String,
            pub restrictions: Option<Map<String, Value>>,
            pub uri: String,
            #[serde(rename = "type")] 
            pub object_type: String,
            $( pub $field: $ty ),*
        }
    };
}

with_album_core_fields!(pub struct AlbumSimple {});

#[derive(Debug, Deserialize, Serialize)]
pub struct Copyright {
    pub text: String,
    #[serde(rename = "type")]
    pub object_type: String,
}

with_album_core_fields!(pub struct AlbumFull {
    pub copyrights: Vec<Copyright>,
    pub external_ids: Map<String, Value>,
    pub genres: Vec<String>,
    pub label: String,
    pub popularity: i32,
    pub tracks: Paging<TrackSimple>,
});

#[derive(Debug, Deserialize, Serialize)]
pub struct AlbumCsv {
    pub origin_artist: String,
    pub album_group: Option<String>,
    pub album_type: String,
    pub id: String,
    pub image_url: Option<String>,
    pub name: String,
    pub release_date: String,
    pub release_date_precision: String,
}

impl AlbumCsv {
    pub fn extract_from(
        album_simple: AlbumSimple,
        origin_artist: String,
    ) -> Self {
        Self {
            origin_artist: origin_artist,
            album_group: album_simple.album_group,
            album_type: album_simple.album_type,
            id: album_simple.id,
            image_url: album_simple.images.get(0).map(|image| image.url.to_owned()),
            name: album_simple.name,
            release_date: album_simple.release_date,
            release_date_precision: album_simple.release_date_precision,
        }
    }
}
