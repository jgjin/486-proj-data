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
};
use csv::{
    Reader,
    Writer,
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

pub fn read_csv_into_sender<D: DeserializeOwned>(
    sender: Sender<D>,
    file_name: &str,
) -> csv::Result<()> {
    let mut reader = Reader::from_path(file_name)?;
    reader.deserialize::<D>().map(|record| {
        let record = record?;
        sender.send(record).unwrap_or_else(|err| {
            println!(
                "Error sending {} data through io::read_csv_into_sender sender: {}",
                file_name,
                err,
            );
        });
        Ok(())
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
