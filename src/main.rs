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
mod client;
mod common_types;
mod io;
mod test;
mod track;
mod track_crawl;
mod track_crawl_2;
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

    let client_ring = Arc::new(RwLock::new(
        client::ClientRing::init(Client::new()).expect("Error in initializing client ring")
    ));

    artist_crawl::artist_crawl_main(100100, client_ring.clone());

    track_crawl_2::track_crawl_main(client_ring);
}
