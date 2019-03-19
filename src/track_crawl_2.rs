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

use crate::{
    artist::{
        get_artist_top_tracks,
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
    track_types::{
        TrackCsv2,
    },
    utils::{
        loop_until_ok,
        SimpleError,
    },
};

fn crawl_artists_tracks_thread(
    artists_crawled: Receiver<ArtistCsv>,
    client_ring: Arc<RwLock<ClientRing>>,
    sender: Sender<TrackCsv2>,
    progress: Arc<ProgressBar>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while let Some(artist_csv) = artists_crawled.recv().ok() {
            loop_until_ok(
                &get_artist_top_tracks,
                client_ring.clone(),
                &artist_csv.id[..],
            ).unwrap_or_else(|err| {
                error!(
                    "Unexpected error in artist::get_artist_top_tracks for {}: {}",
                    artist_csv.id,
                    err,
                );
                vec![]
            }).into_iter().map(|track_full| {
                sender.send(TrackCsv2::extract_from(
                    track_full,
                    &artist_csv,
                )).map_err(|err| SimpleError {
                    message: err.to_string(),
                }.into())
            }).collect::<Result<(), Box<dyn Error>>>().unwrap_or_else(|err| {
                error!(
                    "Error sending {} data through track_crawl_2::crawl_artists_tracks_thread sender: {}",
                    artist_csv.id,
                    err,
                );
            });

            progress.inc(1);
        }
    })
}

pub fn track_crawl(
    artists_crawled: Receiver<ArtistCsv>,
    client_ring: Arc<RwLock<ClientRing>>,
    sender: Sender<TrackCsv2>,
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
        crawl_artists_tracks_thread(
            artists_crawled.clone(),
            client_ring.clone(),
            sender.clone(),
            progress.clone(),
        )
    }).collect();

    threads.into_iter().map(|join_handle| {
        join_handle.join()
    }).collect::<thread::Result<()>>().and_then(|res| {
        progress.finish_with_message("Done crawling tracks");
        Ok(res)
    })
}

#[allow(dead_code)]
pub fn track_crawl_main(
    client_ring: Arc<RwLock<ClientRing>>,
) {
    let (artist_sender, artist_receiver) = channel::unbounded();
    let (track_sender, track_receiver) = channel::unbounded();

    let reader_thread = thread::spawn(move || {
        read_csv_into_sender(artist_sender, "artists_crawled.csv")
            .expect("Error in reading artists crawled")
    });

    let crawler_thread = thread::spawn(move || {
        track_crawl(artist_receiver, client_ring, track_sender)
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
