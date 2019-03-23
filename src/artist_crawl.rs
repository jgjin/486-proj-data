use std::{
    sync::{
        // atomic::{
        //     AtomicUsize,
        //     Ordering,
        // },
        Arc,
        RwLock,
    },
    thread,
};

use atomicring::{
    AtomicRingQueue,
};
use chashmap::{
    CHashMap,
};
use crossbeam_channel::{
    self as channel,
    Sender,
};
use futures::{
    future,
    Future,
};
use indicatif::{
    ProgressBar,
    ProgressStyle,
};
use tokio::{
    runtime::{
        current_thread::{
            Runtime,
        },
    },
};

use crate::{
    artist::{
        get_artist_related_artists,
        search_artists,
    },
    artist_types::{
        ArtistFull,
        ArtistCsv,
    },
    client::{
        ClientRing,
    },
    io::{
        lines_from_file,
        write_csv_through_receiver,
    },
    utils::{
        loop_until_ok,
        SimpleError,
    },
};

fn crawl_related_artists(
    seeds: Vec<ArtistFull>,
    limit: usize,
    client_ring: Arc<RwLock<ClientRing>>,
    sender: Sender<ArtistCsv>,
) {
    let mut rt = Runtime::new().expect("No tokio runtime");
    
    let crawled = Arc::new(CHashMap::new());
    // let num_processed = Arc::new(AtomicUsize::new(0));
    let progress = Arc::new(ProgressBar::new(limit as u64));
    let queue = Arc::new(AtomicRingQueue::with_capacity(limit));

    progress.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{wide_bar}] {pos}/{len} ({percent}%)")
    );
    seeds.into_iter().map(|seed| {
        crawled.insert(seed.id.clone(), ());
        queue.try_push(seed).unwrap_or(());
    }).last();

    let initial_future: Box<Future<Item = (), Error = Box<SimpleError>>> = Box::new(future::ok(()));
    // while num_processed.load(Ordering::SeqCst) < limit {
    let full_future = (0..limit).fold(initial_future, |acc_future, index| {
        let client_ring_clone = client_ring.clone();
        let crawled_clone = crawled.clone();
        // let num_processed_clone = num_processed.clone();
        let progress_clone = progress.clone();
        let queue_clone = queue.clone();
        let sender_clone = sender.clone();

        Box::new(acc_future.join(future::ok(queue.pop()).and_then(|artist| {
            let artist_id_clone = artist.id.clone();

            loop_until_ok(
                &get_artist_related_artists,
                client_ring_clone,
                artist.id.clone(),
            ).map_err(move |err| SimpleError {
                message: format!(
                    "Unexpected error in artist::get_artist_related_artists for {}: {}",
                    artist_id_clone,
                    err,
                ),
            }.into()).map(move |vec| {
                vec.into_iter().map(|artist_full| {
                    if !crawled_clone.contains_key(&artist_full.id) &&
                        !artist_full.genres.is_empty() {
                            crawled_clone.insert(artist_full.id.clone(), ());
                            queue_clone.try_push(artist_full).unwrap_or(());
                        }
                }).last();

                let artist_id = artist.id.clone();
                sender_clone.send(ArtistCsv::from(artist)).unwrap_or_else(|err| {
                    error!(
                        "Error sending {} through artist_crawl::artist_crawl sender: {}",
                        artist_id,
                        err,
                    );
                });
                progress_clone.inc(1);
            })
        })).map(|_| {
            ()
        }))
    });
    // }

    rt.block_on(full_future).unwrap_or_else(|err| {
        error!("Error in running futures: {}", err);
    });
    progress.finish_with_message("Done crawling artists");
}

#[allow(dead_code)]
pub fn artist_crawl_main(
    limit: usize,
    client_ring: Arc<RwLock<ClientRing>>,
) {
    let mut rt = Runtime::new().expect("No tokio runtime");

    let (artist_sender, artist_receiver) = channel::unbounded();

    let seed_artists: Vec<ArtistFull> = lines_from_file("seed_artists.txt")
        .expect("Error in reading seed artists").into_iter().map(|name| {
            rt.block_on(search_artists(client_ring.clone(), name).and_then(|mut vec| {
                vec.items.drain(..).next().ok_or(SimpleError {
                    message: "Empty seed artist".to_string(),
                }.into())
            })).expect("Error in searching artists")
        }).collect();

    let crawler_thread = thread::spawn(move || {
        crawl_related_artists(
            seed_artists,
            limit,
            client_ring,
            artist_sender,
        )
    });

    let writer_thread = thread::spawn(move || {
        write_csv_through_receiver(artist_receiver, "artists_crawled.csv")
            .expect("Error in writing artists");
    });

    crawler_thread.join().unwrap_or_else(|err| {
        error!("Error in artist crawler thread: {:?}", err);
    });

    writer_thread.join().unwrap_or_else(|err| {
        error!("Error in artist writer thread: {:?}", err);
    });

    // info!("fdjkas");
}
