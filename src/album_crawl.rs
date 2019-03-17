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
    self as channel,
    Receiver,
    Sender,
};
use indicatif::{
    ProgressBar,
    ProgressStyle,
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
    client::{
        ClientRing,
    },
    io::{
        lines_from_file,
        read_csv_into_sender,
        write_csv_through_receiver,
    },
    utils::{
        get_next_paging,
        loop_until_ok,
        SimpleError,
    },
};

fn crawl_artists_albums_thread(
    artists_crawled: Receiver<ArtistCsv>,
    client: Arc<Client>,
    client_ring: Arc<RwLock<ClientRing>>,
    sender: Sender<AlbumCsv>,
    progress: Arc<ProgressBar>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while let Some(artist_csv) = artists_crawled.recv().ok() {
            let (mut items, mut next) = loop_until_ok(
                &get_artist_albums,
                client.clone(),
                client_ring.clone(),
                &artist_csv.id[..],
            ).map(|paging| {
                (Some(paging.items), paging.next)
            }).unwrap_or_else(|err| {
                error!(
                    "Unexpected error in artist::get_artist_albums for {}: {}",
                    artist_csv.id,
                    err,
                );
                (None, None)
            });

            while let Some(paging_items) = items {
                paging_items.into_iter().map(|album_simple| {
                    sender.send(
                        AlbumCsv::extract_from(
                            album_simple,
                            artist_csv.id.clone(),
                            artist_csv.genres.clone(),
                        )
                    ).map_err(|err| SimpleError {
                        message: err.to_string(),
                    }.into())
                }).collect::<Result<(), Box<dyn Error>>>().unwrap_or_else(|err| {
                    error!(
                        "Error sending {} data through album_crawl::crawl_artists_album_thread sender: {}",
                        artist_csv.id,
                        err,
                    );
                });

                let (items_new, next_new) = next.map(|next_paging_url| {
                    loop_until_ok(
                        &get_next_paging,
                        client.clone(),
                        client_ring.clone(),
                        &next_paging_url[..],
                    ).map(|paging| {
                        (Some(paging.items), paging.next)
                    }).unwrap_or_else(|err| {
                        error!(
                            "Unexpected error in getting next paging with URL {}: {}",
                            next_paging_url,
                            err,
                        );
                        (None, None)
                    })
                }).unwrap_or((None, None));

                items = items_new;
                next = next_new;
            }

            progress.inc(1);
        }
    })
}

pub fn album_crawl(
    artists_crawled: Receiver<ArtistCsv>,
    client: Arc<Client>,
    client_ring: Arc<RwLock<ClientRing>>,
    sender: Sender<AlbumCsv>,
) -> thread::Result<()> {
    let progress = Arc::new(ProgressBar::new(
        (lines_from_file("artists_crawled.csv")
         .expect("Error in reading artists crawled")
         .len() - 1) as u64
    ));
    progress.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{wide_bar}] {pos}/{len} ({percent}%)")
    );

    let num_threads = num_cpus::get();
    info!("Using {} threads", num_threads);
    
    let threads: Vec<thread::JoinHandle<()>> = (0..num_threads).map(|_| {
        crawl_artists_albums_thread(
            artists_crawled.clone(),
            client.clone(),
            client_ring.clone(),
            sender.clone(),
            progress.clone(),
        )
    }).collect();

    threads.into_iter().map(|join_handle| {
        join_handle.join()
    }).collect::<thread::Result<()>>().and_then(|res| {
        progress.finish_with_message("Done crawling albums");
        Ok(res)
    })
}

#[allow(dead_code)]
pub fn album_crawl_main(
    client: Arc<Client>,
    client_ring: Arc<RwLock<ClientRing>>,
) {
    let (artist_sender, artist_receiver) = channel::unbounded();
    let (album_sender, album_receiver) = channel::unbounded();

    let reader_thread = thread::spawn(move || {
        read_csv_into_sender(artist_sender, "artists_crawled.csv")
            .expect("Error in reading artists crawled")
    });

    let crawler_thread = thread::spawn(move || {
        album_crawl(artist_receiver, client, client_ring, album_sender)
            .expect("Error in crawling tracks");
    });

    let writer_thread = thread::spawn(move || {
        write_csv_through_receiver(album_receiver, "albums_crawled.csv")
            .expect("Error in writing tracks");
    });

    reader_thread.join().unwrap_or_else(|err| {
        error!("Error in album reader thread: {:?}", err);
    });

    crawler_thread.join().unwrap_or_else(|err| {
        error!("Error in album crawler thread: {:?}", err);
    });

    writer_thread.join().unwrap_or_else(|err| {
        error!("Error in album writer thread: {:?}", err);
    });
}
