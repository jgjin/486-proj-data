use std::{
    sync::{
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
use num_cpus;
use reqwest::{
    Client,
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
    io::{
        lines_from_file,
        write_csv_through_receiver,
    },
};

fn crawl_related_artists_thread(
    queue: Arc<SegQueue<ArtistFull>>,
    crawled: Arc<CHashMap<String, ()>>,
    limit: usize,
    client: Arc<Client>,
    token: Arc<RwLock<String>>,
    sender: Sender<ArtistCsv>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut sleep_period = 1;

        while crawled.len() < limit {
            queue.pop().map(|artist| {
                get_artist_related_artists(
                    client.clone(),
                    token.clone(),
                    &artist.id[..],
                ).unwrap_or_else(|err| {
                    error!(
                        "Error in artist::get_artist_related_artists for {}: {}",
                        artist.id,
                        err,
                    );
                    vec![]
                }).into_iter().filter(|artist_full| {
                    !crawled.contains_key(&artist_full.id)
                }).map(|artist_full| {
                    queue.push(artist_full);
                }).last();

                let artist_id = artist.id.clone();
                // Avoids extra last-minute inserts when limit reached
                if crawled.len() < limit {
                    sender.send(ArtistCsv::from(artist)).unwrap_or_else(|err| {
                        error!(
                            "Error sending {} through artist_crawl::artist_crawl sender: {}",
                            artist_id,
                            err,
                        );
                    });
                    crawled.insert(artist_id, ());
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
    client: Arc<Client>,
    token: Arc<RwLock<String>>,
    sender: Sender<ArtistCsv>,
) -> thread::Result<CHashMap<String, ()>> {
    let queue = Arc::new(SegQueue::new());
    seeds.into_iter().map(|seed| {
        queue.push(seed);
    }).last();
    let crawled = Arc::new(CHashMap::new());
    
    let num_threads = num_cpus::get();
    info!("Using {} threads", num_threads);

    let threads: Vec<thread::JoinHandle<()>> = (0..num_threads).map(|_| {
        crawl_related_artists_thread(
            queue.clone(),
            crawled.clone(),
            limit,
            client.clone(),
            token.clone(),
            sender.clone(),
        )
    }).collect();

    threads.into_iter().map(|join_handle| {
        join_handle.join()
    }).collect::<thread::Result<()>>().and_then(|_| {
        Ok(Arc::try_unwrap(crawled).expect("Error in unwrapping Arc for artist_crawl"))
    })
}

#[allow(dead_code)]
pub fn artist_crawl_main(
    limit: usize,
    client: Arc<Client>,
    token: Arc<RwLock<String>>,
) {
    let (artist_sender, artist_receiver) = channel::unbounded();

    let seed_artists: Vec<ArtistFull> = lines_from_file("seed_artists.txt")
        .expect("Error in reading seed artists").into_iter().map(|name| {
            search_artists(client.clone(), token.clone(), &name[..])
                .expect("Error in searching artists")
                .items.drain(..).next().expect("No artists found")
        }).collect();

    let crawler_thread = thread::spawn(move || {
        artist_crawl(
            seed_artists,
            limit,
            client,
            token,
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
