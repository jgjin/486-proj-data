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
