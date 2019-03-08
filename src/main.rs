extern crate atomicring;
extern crate chashmap;
extern crate crossbeam_channel;
extern crate crossbeam_queue;
extern crate csv;
extern crate indicatif;
extern crate itertools;
#[macro_use] extern crate log;
extern crate num_cpus;
extern crate pretty_env_logger;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

mod album;
mod album_crawl;
mod album_types;
mod artist;
mod artist_crawl;
mod artist_types;
mod common_types;
mod io;
mod test;
mod token;
mod track;
mod track_crawl;
mod track_types;
mod utils;

use std::{
    sync::{
        Arc,
        RwLock,
    },
};

use reqwest::{
    Client,
};

fn main(
) {
    pretty_env_logger::init();
    
    let client = Arc::new(Client::new());

    let token = Arc::new(RwLock::new(token::TokenRing::init(client.clone())));

    artist_crawl::artist_crawl_main(20, client.clone(), token.clone());

    album_crawl::album_crawl_main(client.clone(), token.clone());
    
    track_crawl::track_crawl_main(client, token);
}
