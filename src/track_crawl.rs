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
    album::{
        get_albums,
    },
    album_types::{
        AlbumCsv,
    },
    io::{
        lines_from_file,
        read_csv_chunks_into_sender,
        write_csv_through_receiver,
    },
    track_types::{
        TrackCsv,
    },
    utils::{
        get_next_paging,
        SimpleError,
    },
};

struct NextPaging {
    origin_album: String,
    url: String,
}

fn crawl_albums_tracks_thread(
    albums_crawled: Receiver<Vec<AlbumCsv>>,
    client: Arc<Client>,
    token: Arc<RwLock<String>>,
    sender: Sender<TrackCsv>,
    progress: Arc<ProgressBar>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while let Some(albums_csv) = albums_crawled.recv().ok() {
            let mut next_pagings = Vec::new();

            let albums_ids = albums_csv.iter().map(|album_csv| {
                &album_csv.id[..]
            }).collect();

            get_albums(
                client.clone(),
                token.clone(),
                albums_ids,
            ).unwrap_or_else(|err| {
                error!(
                    "Error in album::get_albums : {}",
                    err,
                );
                vec![]
            }).into_iter().map(|album_full| {
                let album_id = album_full.id.clone();
                album_full.tracks.next.map(|next_url| {
                    next_pagings.push(NextPaging {
                        origin_album: album_id.clone(),
                        url: next_url.clone(),
                    });
                });
                album_full.tracks.items.into_iter().map(|track_simple| {
                    sender.send(TrackCsv::extract_from(track_simple, album_id.clone()))
                        .map_err(|err| SimpleError {
                            message: err.to_string(),
                        }.into())
                }).collect::<Result<(), Box<dyn Error>>>().unwrap_or_else(|err| {
                    error!(
                        "Error sending {} data through track_crawl::crawl_albums_tracks_thread sender: {}",
                        album_id,
                        err,
                    );
                });
            }).last();

            while let Some(next_paging) = next_pagings.pop() {
                let origin_album = next_paging.origin_album.clone();
                get_next_paging(&next_paging.url[..], client.clone(), token.clone())
                    .map(|paging| {
                        paging.next.map(|next_url| {
                            next_pagings.push(NextPaging{
                                origin_album: origin_album.clone(),
                                url: next_url,
                            });
                        });
                        paging.items.into_iter().map(|track_simple| {
                            sender.send(TrackCsv::extract_from(track_simple, origin_album.clone()))
                                .map_err(|err| SimpleError {
                                    message: err.to_string(),
                                }.into())
                        }).collect::<Result<(), Box<dyn Error>>>().unwrap_or_else(|err| {
                            error!(
                                "Error sending {} data through track_crawl::crawl_albums_tracks_thread sender: {}",
                                origin_album,
                                err,
                            );
                        });
                    }).unwrap_or_else(|err| {
                        error!(
                            "Error getting next paging with URL {}: {}",
                            next_paging.url,
                            err,
                        );
                    });
            }

            progress.inc(1);
        }
    })
}

pub fn track_crawl(
    albums_crawled: Receiver<Vec<AlbumCsv>>,
    client: Arc<Client>,
    token: Arc<RwLock<String>>,
    sender: Sender<TrackCsv>,
) -> thread::Result<()> {
    let progress = Arc::new(ProgressBar::new(
        (lines_from_file("albums_crawled.csv")
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
        crawl_albums_tracks_thread(
            albums_crawled.clone(),
            client.clone(),
            token.clone(),
            sender.clone(),
            progress.clone(),
        )
    }).collect();

    threads.into_iter().map(|join_handle| {
        join_handle.join()
    }).collect()
}

#[allow(dead_code)]
pub fn track_crawl_main(
    client: Arc<Client>,
    token: Arc<RwLock<String>>,
) {
    let (album_sender, album_receiver) = channel::unbounded();
    let (track_sender, track_receiver) = channel::unbounded();
    
    let reader_thread = thread::spawn(move || {
        read_csv_chunks_into_sender(2, album_sender, "albums_crawled.csv")
            .expect("Error in reading albums crawled")
    });

    let crawler_thread = thread::spawn(move || {
        track_crawl(album_receiver, client, token, track_sender)
            .expect("Error in crawling tracks");
    });

    let writer_thread = thread::spawn(move || {
        write_csv_through_receiver(track_receiver, "tracks_crawled.csv")
            .expect("Error in writing tracks");
    });

    reader_thread.join().unwrap_or_else(|err| {
        error!("Error in track reader thread: {:?}", err);
    });

    crawler_thread.join().unwrap_or_else(|err| {
        error!("Error in track crawler thread: {:?}", err);
    });

    writer_thread.join().unwrap_or_else(|err| {
        error!("Error in track writer thread: {:?}", err);
    });
}
