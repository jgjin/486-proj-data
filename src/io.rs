use std::{
    fs::{
        File,
    },
    io::{
        BufRead,
        BufReader,
    },
};

use crossbeam_channel::{
    Receiver,
    Sender,
    SendError
};
use csv::{
    Reader,
    Writer,
};
use itertools::{
    Itertools
};
use serde::{
    Serialize,
    de::{
        DeserializeOwned,
    },
};

#[allow(dead_code)]
pub fn lines_from_file(
    file_name: &str,
) -> std::io::Result<Vec<String>> {
    Ok(
        BufReader::new(File::open(file_name)?).lines().map(|line| {
            line.expect("Error in reading line")
        }).collect()
    )
}

pub fn structs_from_file<D: DeserializeOwned>(
    file_name: &str,
) -> csv::Result<Vec<D>> {
    let mut reader = Reader::from_path(file_name)?;
    reader.deserialize::<D>().map(|record| {
        Ok(record?)
    }).collect()
}

pub fn read_csv_into_sender<D: DeserializeOwned>(
    sender: Sender<D>,
    file_name: &str,
) -> Result<(), SendError<D>> {
    let mut reader = Reader::from_path(file_name).expect("Error opening reader");
    reader.deserialize::<D>().map(|record| {
        let record = record.expect("Error reading record");
        sender.send(record)
    }).collect()
}

pub fn read_csv_chunks_into_sender<D: DeserializeOwned>(
    chunk_size: usize,
    sender: Sender<Vec<D>>,
    file_name: &str,
) -> Result<(), SendError<Vec<D>>> {
    let mut reader = Reader::from_path(file_name).expect("Error opening reader");
    reader.deserialize::<D>().map(|record| {
        record.expect("Error reading record")
    }).chunks(chunk_size).into_iter().map(|record_chunk| {
        sender.send(record_chunk.collect())
    }).collect()
}

pub fn write_csv_through_receiver<S: Serialize>(
    receiver: Receiver<S>,
    file_name: &str,
) -> csv::Result<()> {
    let mut writer = Writer::from_path(file_name)?;

    while let Some(record) = receiver.recv().ok() {
        writer.serialize(record)?
    }

    Ok(())
}
