use std::{
    error::{
        Error,
    },
    sync::{
        Arc,
        RwLock,
    },
    thread,
};

use crossbeam_channel::{
    Receiver,
    Sender,
};
use num_cpus;
use reqwest::{
    Client,
};

use crate::{
    album_types::{
        AlbumCsv,
    },
    artist::{
        get_artist_albums,
    },
    artist_types::{
        ArtistCsv,
    },
    utils::{
        get_next_paging,
        SimpleError,
    },
};

fn crawl_artists_album_thread(
    artists_crawled: Receiver<ArtistCsv>,
    client: Arc<Client>,
    token: Arc<RwLock<String>>,
    sender: Sender<AlbumCsv>,
) -> thread::Result<()> {
    thread::spawn(move || {
        while let Some(artist_csv) = artists_crawled.recv().ok() {
            let (mut items, mut next) = get_artist_albums(
                client.clone(),
                token.clone(),
                &artist_csv.id[..],
            ).map(|paging| {
                (Some(paging.items), paging.next)
            }).unwrap_or_else(|err| {
                println!(
                    "Error in artist::get_artist_albums for {}: {}",
                    artist_csv.id,
                    err,
                );
                (None, None)
            });

            while let Some(paging_items) = items {
                paging_items.into_iter().map(|album_simple| {
                    sender.send(
                        AlbumCsv::extract_from(album_simple, artist_csv.id.clone())
                    ).map_err(|err| SimpleError {
                        message: err.to_string(),
                    }.into())
                }).collect::<Result<(), Box<dyn Error>>>().unwrap_or_else(|err| {
                    println!(
                        "Error sending {} data through album_crawl::crawl_artists_album_thread sender: {}",
                        artist_csv.id,
                        err
                    );
                });

                let (items_new, next_new) = next.map(|next_paging_url| {
                    get_next_paging(&next_paging_url[..], client.clone(), token.clone())
                        .map(|paging| {
                            (Some(paging.items), paging.next)
                        }).unwrap_or_else(|err| {
                            println!(
                                "Error getting next paging with URL {}: {}",
                                next_paging_url,
                                err,
                            );
                            (None, None)
                        })
                }).unwrap_or((None, None));

                items = items_new;
                next = next_new;
            }
        }
    }).join()
}

pub fn album_crawl(
    artists_crawled: Receiver<ArtistCsv>,
    client: Arc<Client>,
    token: Arc<RwLock<String>>,
    sender: Sender<AlbumCsv>,
) -> thread::Result<()> {
    let num_threads = num_cpus::get();
    println!("Using {} threads", num_threads);

    (0..num_threads).map(|_| {
        crawl_artists_album_thread(
            artists_crawled.clone(),
            client.clone(),
            token.clone(),
            sender.clone(),
        )
    }).collect()
}
