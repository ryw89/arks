use bzip2::read::BzDecoder;
use colored::*;
use flate2::read::GzDecoder;
use std::fs::File;
use tar::Archive;

use crate::archivetypes::ArchiveEntry;
use crate::file_types::is_plain_text;
use crate::search::{check_inner_file_pattern, search_entry};

pub trait UnpackAndSearch {
    fn unpack_and_search(&mut self, text: &str, path: &str);
}

impl UnpackAndSearch for Archive<GzDecoder<File>> {
    fn unpack_and_search(&mut self, text: &str, path: &str) {
        self.entries().unwrap().for_each(|x| match x {
            Ok(e) => unpack_and_search_entry(ArchiveEntry::GzEntry(Box::new(e)), text),
            Err(_) => print_extract_entry_err(path),
        });
    }
}

impl UnpackAndSearch for Archive<BzDecoder<File>> {
    fn unpack_and_search(&mut self, text: &str, path: &str) {
        self.entries().unwrap().for_each(|x| match x {
            Ok(e) => unpack_and_search_entry(ArchiveEntry::Bz2Entry(Box::new(e)), text),
            Err(_) => print_extract_entry_err(path),
        });
    }
}

fn print_extract_entry_err(path: &str) {
    eprintln!(
        "{}",
        format!("Warning: Could not extract an entry in {}", path).red()
    );
}

pub fn unpack_and_search_entry(entry: ArchiveEntry, text: &str) {
    let err_msg = "ArchiveEntry enum variant must be Bz2Entry or GzEntry.".to_string();

    let path = match entry {
        ArchiveEntry::Bz2Entry(ref e) => e.path().unwrap().display().to_string(),
        ArchiveEntry::GzEntry(ref e) => e.path().unwrap().display().to_string(),
        _ => panic!("{}", err_msg),
    };

    // Early return if pattern does not match
    if !check_inner_file_pattern(&path) {
        return;
    }

    if is_plain_text(&path) {
        match entry {
            ArchiveEntry::Bz2Entry(e) => search_entry(ArchiveEntry::Bz2Entry(e), text),
            ArchiveEntry::GzEntry(e) => search_entry(ArchiveEntry::GzEntry(e), text),
            _ => panic!("{}", err_msg),
        }
    }
}

pub fn unpack_and_search_tar<T: UnpackAndSearch>(mut archive: T, text: &str, path: &str) {
    archive.unpack_and_search(text, path)
}
