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
};
use csv::{
    Writer,
};
use serde::{
    Serialize,
};

pub fn lines_from_file(
    file_name: &str,
) -> std::io::Result<Vec<String>> {
    Ok(
        BufReader::new(File::open(file_name)?).lines().map(|line| {
            line.expect("Error in reading line")
        }).collect()
    )
}

pub fn write_csv_through_receiver<S: Serialize>(
    receiver: Receiver<S>,
    limit: usize,
    file_name: &str,
) -> csv::Result<()> {
    let mut writer = Writer::from_path(file_name)?;

    (0..limit).map(|_| {
        receiver.recv().map(|record| {
            writer.serialize(record)
        }).unwrap_or(Ok(())) // None implies channel is closed and empty, so we handle as Ok
    }).collect()
}
