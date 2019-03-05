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
    },
    artist_types::{
        ArtistCsv,
    },
};

fn crawl_related_artists_thread(
    queue: Arc<SegQueue<String>>,
    crawled: Arc<CHashMap<String, ()>>,
    limit: usize,
    client: Arc<Client>,
    token: Arc<RwLock<String>>,
    sender: Sender<ArtistCsv>,
) -> thread::Result<()> {
    thread::spawn(move || {
        let mut sleep_period = 1;

        while crawled.len() < limit {
            queue.pop().map(|query_id| {
                get_artist_related_artists(
                    client.clone(),
                    token.clone(),
                    &query_id[..],
                ).unwrap_or_else(|err| {
                    println!(
                        "Error in artist::get_artist_related_artists for {}: {}",
                        query_id,
                        err,
                    );
                    vec![]
                }).into_iter().filter(|artist_full| {
                    !crawled.contains_key(&artist_full.id)
                }).map(|artist_full| {
                    let artist_id = artist_full.id.clone();
                    sender.send(ArtistCsv::from(artist_full)).unwrap_or_else(|err| {
                        println!(
                            "Error sending {} through artist_crawl::artist_crawl sender: {}",
                            artist_id,
                            err,
                        );
                    });
                    crawled.insert(artist_id.clone(), ());
                    queue.push(artist_id);
                }).last();
            }).unwrap_or_else(|_| {
                if sleep_period > 8 {
                    panic!("Timed out before 0 remaining");
                }

                thread::sleep(Duration::from_secs(sleep_period));
                sleep_period *= 2;
            })
        }
    }).join()
}

#[allow(dead_code)]
pub fn artist_crawl(
    seeds: Vec<&str>,
    limit: usize,
    client: Arc<Client>,
    token: Arc<RwLock<String>>,
    sender: Sender<ArtistCsv>,
) -> CHashMap<String, ()> {
    let queue = Arc::new(SegQueue::new());
    seeds.into_iter().map(|seed| {
        queue.push(seed.to_string());
    }).last();
    let crawled = Arc::new(CHashMap::new());
    
    let num_threads = num_cpus::get();
    println!("Using {} threads", num_threads);

    let result: thread::Result<()> = (0..num_threads).map(|_| {
        crawl_related_artists_thread(
            queue.clone(),
            crawled.clone(),
            limit,
            client.clone(),
            token.clone(),
            sender.clone(),
        )
    }).collect();

    result.unwrap_or_else(|err| {
        println!("Error in artist_crawl::artist_crawl: {:?}", err);
    });

    Arc::try_unwrap(crawled).expect("Error in unwrapping Arc")
}
