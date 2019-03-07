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
    pub popularity: Option<i32>,
});

#[derive(Debug, Deserialize, Serialize)]
pub struct ArtistCsv {
    pub id: String,
    pub name: String,
    // pub followers_total: i32,
    // pub genres: String,
    // pub image_url: Option<String>,
    pub popularity: i32,
}

impl From<ArtistFull> for ArtistCsv {
    fn from(
        artist_full: ArtistFull,
    ) -> Self {
        Self {
            id: artist_full.id,
            name: artist_full.name,
            // followers_total: artist_full.followers.total,
            // genres: artist_full.genres.join(", "),
            // image_url: artist_full.images.get(0).map(|image| image.url.to_owned()),
            popularity: artist_full.popularity.unwrap_or(-1),
        }
    }
}
