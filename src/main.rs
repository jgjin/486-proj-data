extern crate chashmap;
extern crate crossbeam_channel;
extern crate crossbeam_queue;
extern crate csv;
extern crate num_cpus;
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
mod track_types;
mod utils;

use std::{
    sync::{
        Arc,
        RwLock,
    },
    thread,
};

use crossbeam_channel as channel;
use reqwest::{
    Client,
};

fn main(
) {
    let client = Arc::new(Client::new());

    let token = Arc::new(RwLock::new(
        token::retrieve_access_token(client.clone())
            .expect("Error in access token")
            .access_token
    ));

    let (artist_sender, artist_receiver) = channel::unbounded();
    let (album_sender, album_receiver) = channel::unbounded();
    

    let reader_thread = thread::spawn(move || {
        io::read_csv_into_sender::<artist_types::ArtistCsv>(
            artist_sender,
            "artists_crawled.csv",
        ).unwrap_or_else(|err| {
            println!("Error in reading csv: {}", err)
        });
        // println!("Finished reading");
    });

    let crawler_thread = thread::spawn(move || {
        album_crawl::album_crawl(
            artist_receiver,
            client,
            token,
            album_sender
        ).unwrap_or_else(|err| {
            println!("Error in crawling albums: {:?}", err)
        });
        // println!("Finished crawling");
    });

    let writer_thread = thread::spawn(move || {
        io::write_csv_through_receiver(album_receiver, "albums_crawled.csv")
            .unwrap_or_else(|err| {
                println!("Error in writing csv: {}", err)
            });
        // println!("Finished writing");
    });

    reader_thread.join().unwrap_or_else(|err| {
        println!("Error in csv reader thread: {:?}", err);
    });

    crawler_thread.join().unwrap_or_else(|err| {
        println!("Error in album crawler thread: {:?}", err);
    });

    writer_thread.join().unwrap_or_else(|err| {
        println!("Error in csv writer thread: {:?}", err);
    });
}
