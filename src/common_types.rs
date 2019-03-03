use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Image {
    pub height: i32,
    pub url: String,
    pub width: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Paging<Item> {
    pub href: String,
    pub items: Vec<Item>,
    pub limit: i32,
    pub next: Option<String>,
    pub offset: i32,
    pub previous: Option<String>,
    pub total: i32,
}
