use std::{
    sync::{
        atomic::{
            AtomicUsize,
            Ordering,
        },
        Arc,
        RwLock,
    },
    thread,
    time::{
        Duration,
    },
};

use chashmap::{
    CHashMap,
};
use crossbeam_channel::{
    self as channel,
    Sender,
};
use crossbeam_queue::{
    SegQueue,
};
use indicatif::{
    ProgressBar,
    ProgressStyle,
};
use num_cpus;

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
    },
};

fn crawl_related_artists_thread(
    queue: Arc<SegQueue<ArtistFull>>,
    crawled: Arc<CHashMap<String, ()>>,
    num_processed: Arc<AtomicUsize>,
    limit: usize,
    client_ring: Arc<RwLock<ClientRing>>,
    sender: Sender<ArtistCsv>,
    progress: Arc<ProgressBar>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut sleep_period = 1;

        while num_processed.load(Ordering::SeqCst) < limit {
            queue.pop().map(|artist| {
                loop_until_ok(
                    &get_artist_related_artists,
                    client_ring.clone(),
                    &artist.id[..],
                ).unwrap_or_else(|err| {
                    error!(
                        "Unexpected error in artist::get_artist_related_artists for {}: {}",
                        artist.id,
                        err,
                    );
                    vec![]
                }).into_iter().map(|artist_full| {
                    if !crawled.contains_key(&artist_full.id) {
                        crawled.insert(artist_full.id.clone(), ());
                        queue.push(artist_full);
                    }
                }).last();

                let artist_id = artist.id.clone();
                // Avoids extra last-minute inserts when limit reached
                // Still not exact, may insert extra entries?
                if num_processed.load(Ordering::SeqCst) < limit {
                    sender.send(ArtistCsv::from(artist)).unwrap_or_else(|err| {
                        error!(
                            "Error sending {} through artist_crawl::artist_crawl sender: {}",
                            artist_id,
                            err,
                        );
                    });
                    num_processed.fetch_add(1, Ordering::SeqCst);
                    progress.inc(1);
                }
            }).unwrap_or_else(|_| {
                if sleep_period > 8 {
                    panic!("Timed out before 0 remaining");
                }

                thread::sleep(Duration::from_secs(sleep_period));
                sleep_period *= 2;
            })
        }
    })
}

pub fn artist_crawl(
    seeds: Vec<ArtistFull>,
    limit: usize,
    client_ring: Arc<RwLock<ClientRing>>,
    sender: Sender<ArtistCsv>,
) -> thread::Result<CHashMap<String, ()>> {
    let queue = Arc::new(SegQueue::new());
    let crawled = Arc::new(CHashMap::new());
    seeds.into_iter().map(|seed| {
        crawled.insert(seed.id.clone(), ());
        queue.push(seed);
    }).last();
    let num_processed = Arc::new(AtomicUsize::new(0));
    let progress = Arc::new(ProgressBar::new(limit as u64));
    progress.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{wide_bar}] {pos}/{len} ({percent}%)")
    );
    
    let num_threads = num_cpus::get();
    info!("Using {} threads", num_threads);

    let threads: Vec<thread::JoinHandle<()>> = (0..num_threads).map(|_| {
        crawl_related_artists_thread(
            queue.clone(),
            crawled.clone(),
            num_processed.clone(),
            limit,
            client_ring.clone(),
            sender.clone(),
            progress.clone(),
        )
    }).collect();

    threads.into_iter().map(|join_handle| {
        join_handle.join()
    }).collect::<thread::Result<()>>().and_then(|_| {
        progress.finish_with_message("Done crawling artists");
        Ok(Arc::try_unwrap(crawled).expect("Error in unwrapping Arc for artist_crawl"))
    })
}

#[allow(dead_code)]
pub fn artist_crawl_main(
    limit: usize,
    client_ring: Arc<RwLock<ClientRing>>,
) {
    let (artist_sender, artist_receiver) = channel::unbounded();
    
    let seed_artists: Vec<ArtistFull> = lines_from_file("seed_artists.txt")
        .expect("Error in reading seed artists").into_iter().map(|name| {
            search_artists(client_ring.clone(), &name[..])
                .expect("Error in searching artists")
                .items.drain(..).next().expect("No artists found")
        }).collect();

    let crawler_thread = thread::spawn(move || {
        artist_crawl(
            seed_artists,
            limit,
            client_ring,
            artist_sender,
        ).expect("Error in crawling artists");
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
}
