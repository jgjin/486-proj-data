extern crate chashmap;
extern crate crossbeam_channel;
extern crate crossbeam_queue;
extern crate csv;
extern crate num_cpus;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

mod album;
mod album_types;
mod artist;
mod artist_crawl;
mod artist_types;
mod common_types;
mod io;
mod test;
mod token;
mod track;
mod track_types;
mod utils;

use std::{
    sync::{
        Arc,
    },
};

use crossbeam_channel as channel;
use reqwest::{
    Client,
};

fn main(
) {
    let client = Arc::new(Client::new());

    let token = Arc::new(
        token::retrieve_access_token(&client)
            .expect("Error in access token")
            .access_token
    );

    let seed_artists: Vec<String> = io::lines_from_file("seed_artists.txt")
        .expect("Error in reading file")
        .into_iter().map(|query| {
            artist::search_artists(client.clone(), token.clone(), &query[..])
                .expect("Error in artist::search_artists")
                .items[0].id.to_owned()
        }).collect();


    let (sender, receiver) = channel::unbounded();
    let limit = 200000;

    artist_crawl::artist_crawl(
        seed_artists.iter().map(|artist_id| &artist_id[..]).collect(),
        limit,
        client,
        token,
        sender,
    );
    
    io::write_csv_through_receiver(receiver, limit, "artists_crawled.csv")
        .expect("Error in writing csv");
}
