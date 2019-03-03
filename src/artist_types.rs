use serde::{
    Deserialize,
    Serialize,
};
use serde_json::{
    Map,
    Value,
};

use crate::{
    common_types::{
        Image,
    },
};

macro_rules! with_artist_core_fields {
    (pub struct $name:ident { $( pub $field:ident: $ty:ty ),* $(,)* }) => {
        #[derive(Debug, Deserialize, Serialize)]
        pub struct $name {
            pub external_urls: Map<String, Value>,
            pub href: String,
            pub id: String,
            pub name: String,
            pub uri: String,
            #[serde(rename = "type")] 
            pub object_type: String,
            $( pub $field: $ty ),*
        }
    };
}

with_artist_core_fields!(pub struct ArtistSimple {});

#[derive(Debug, Deserialize, Serialize)]
pub struct Followers {
    href: Option<String>,
    total: i32,
}

with_artist_core_fields!(pub struct ArtistFull {
    pub followers: Followers,
    pub genres: Vec<String>,
    pub images: Vec<Image>,
    pub popularity: i32,
});

#[derive(Debug, Deserialize, Serialize)]
pub struct ArtistCsv {
    pub href: String,
    pub id: String,
    pub name: String,
    pub uri: String,
}

impl From<ArtistFull> for ArtistCsv {
    fn from(artist_full: ArtistFull) -> Self {
        Self {
            href: artist_full.href,
            id: artist_full.id,
            name: artist_full.name,
            uri: artist_full.uri,
        }
    }
}
