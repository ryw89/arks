use colored::*;
use flate2::read::GzDecoder;
use std::fs::File;
use tar::{Archive, Entry};

use crate::archivetypes::ArchiveEntry;
use crate::file_types::is_plain_text;
use crate::search::{check_inner_file_pattern, search_entry};

pub fn unpack_and_search_gz_entry(
    entry: Entry<GzDecoder<File>>,
    text: &str,
    inner_pattern: Option<&String>,
) {
    // Infer MIME type
    let path = entry.path().unwrap().display().to_string();

    if !check_inner_file_pattern(&path, inner_pattern) {}

    if is_plain_text(&path) {
        search_entry(ArchiveEntry::GzEntry(Box::new(entry)), text);
    }
}

pub fn unpack_and_search_targz(
    mut archive: Archive<GzDecoder<File>>,
    text: &str,
    path: &str,
    inner_pattern: Option<String>,
) {
    archive.entries().unwrap().for_each(|x| match x {
        Ok(e) => unpack_and_search_gz_entry(e, text, inner_pattern.as_ref()),
        Err(_) => {
            eprintln!("Warning: Could not extract an entry in {}", path.red());
        }
    });
}
