use colored::*;
use std::fs::read_to_string;
use std::io::{self, Read, Write};
use tempfile::NamedTempFile;

use crate::archivetypes::ArchiveEntry;

fn extract_entry_text_and_path(entry: ArchiveEntry) -> (String, String) {
    // Unpack entry to the filesystem
    let file = NamedTempFile::new().unwrap();

    match entry {
        ArchiveEntry::GzEntry(mut e) => {
            e.unpack(&file).unwrap();

            // Get entry path
            let path = e.path().unwrap().display().to_string();

            // Read into string
            let contents = read_to_string(&file.path()).unwrap();

            (contents, path)
        }
        ArchiveEntry::ZipFile(mut e) => {
            let path = e.name().to_string();
            let mut contents = String::new();
            e.read_to_string(&mut contents).unwrap();

            (contents, path)
        }
    }
}

pub fn search_entry(entry: ArchiveEntry, text: &str) {
    // Extract text of entry
    let (contents, path) = match entry {
        ArchiveEntry::GzEntry(e) => extract_entry_text_and_path(ArchiveEntry::GzEntry(e)),
        ArchiveEntry::ZipFile(e) => extract_entry_text_and_path(ArchiveEntry::ZipFile(e)),
    };

    contents.lines().enumerate().for_each(|(i, line)| {
        if line.contains(text) {
            print!("{}:{}:", path.red(), (i + 1).to_string().yellow());

            for s in line.split_inclusive(text) {
                if s.contains(text) {
                    let split = s.split(text).collect::<Vec<&str>>();
                    if split.len() != 2 {
                        panic!("Expected vector of length 2.");
                    }
                    let other_bit = split[0];

                    // Check if text substring is at beginning or not
                    let pos = s.find(text).unwrap();
                    if pos == 0 {
                        print!("{}", text.red());
                        print!("{}", other_bit);
                    } else {
                        print!("{}", other_bit);
                        print!("{}", text.red());
                    }
                } else {
                    print!("{}", s);
                }
            }
            println!();
            io::stdout().flush().unwrap();
        }
    });
}
