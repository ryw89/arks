use colored::*;
use flate2::read::GzDecoder;
use std::fs::File;
use tar::{Archive, Entry};

use crate::archivetypes::ArchiveEntry;
use crate::search::search_entry;

pub fn unpack_and_search_gz_entry(entry: Entry<GzDecoder<File>>, text: &str) {
    // Infer MIME type
    let path = entry.path().unwrap().display().to_string();
    match mime_guess::from_path(&path).first_raw() {
        None => {
            // Occurs for directories & extensionless files
        }
        Some(m) => {
            if m.starts_with("text/") {
                search_entry(ArchiveEntry::GzEntry(entry), text);
            } else {
            }
        }
    }
}

pub fn unpack_and_search_targz(mut archive: Archive<GzDecoder<File>>, text: &str, path: &str) {
    archive.entries().unwrap().for_each(|x| match x {
        Ok(e) => unpack_and_search_gz_entry(e, text),
        Err(_) => {
            eprintln!("Warning: Could not extract an entry in {}", path.red());
        }
    });
}
