use cached::proc_macro::cached;
use colored::*;
use std::fs::read_to_string;
use std::io::{self, ErrorKind, Read, Write};
use std::sync::Mutex;
use tar::Entry;
use tempfile::NamedTempFile;

use crate::{archivetypes::ArchiveEntry, CONFIG};

#[cached]
fn get_inner_pattern() -> Option<Vec<String>> {
    let guard = CONFIG.inner_pattern.lock().unwrap();
    let pattern: Option<String> = guard.clone();
    Mutex::unlock(guard);

    // A None will stay a None, so just return
    if pattern.is_none() {
        pattern.as_ref()?;
    }

    // Otherwise, split out the string into a Vec<String>. Note that
    // the user may optional indicate multiple patterns to match on by
    // delimiting with '|'.
    let out: Vec<String> = pattern.unwrap().split('|').map(|s| s.to_string()).collect();

    Some(out)
}

pub fn check_inner_file_pattern(path: &str) -> bool {
    let pattern = get_inner_pattern();
    match pattern {
        None => true,
        Some(p) => {
            for c in p {
                if path.contains(&c) {
                    return true;
                }
            }
            false
        }
    }
}

fn make_utf8_err_str(path: &str) -> ColoredString {
    format!("Error: {} does not appear to be valid UTF-8.", path).red()
}

/// Extract a gzip or bzip2 compressed tar entry.
///
/// If the tar entry contains valid UTF-8 text, this function will
/// return a tuple of the entry's contents as a String and also its
/// path within the tar archive as a String. Or, if the entry is not
/// UTF-8 text, an empty string will be returned as the contents (and
/// the path will also be returned, as before). As a side effect,
/// non-UTF-8 entries will result in an message being printed to
/// stderr.
fn extract_compressed_tar_entry<T: Read>(mut e: Box<Entry<T>>) -> (String, String) {
    // Unpack entry to the filesystem
    let file = NamedTempFile::new().unwrap();

    e.unpack(&file).unwrap();

    // Get entry path
    let path = e.path().unwrap().display().to_string();

    // Read into string
    match read_to_string(&file.path()) {
        Ok(s) => (s, path),
        Err(e) => match e.kind() {
            ErrorKind::InvalidData => {
                eprintln!("{}", make_utf8_err_str(&path));
                // While returning an empty string rather than
                // an Option may not be idiomatic, an empty
                // will string will simply just not have any
                // matches, which is the correct behavior in
                // the case of non-UTF8 data
                ("".to_string(), path)
            }
            _ => panic!("{}", e),
        },
    }
}

fn extract_entry_text_and_path(entry: ArchiveEntry) -> (String, String) {
    match entry {
        ArchiveEntry::Bz2Entry(e) => extract_compressed_tar_entry(e),
        ArchiveEntry::GzEntry(e) => extract_compressed_tar_entry(e),
        ArchiveEntry::ZipFile(mut e) => {
            let path = e.name().to_string();
            let mut contents = String::new();
            match e.read_to_string(&mut contents) {
                Ok(_) => {}
                Err(e) => match e.kind() {
                    ErrorKind::InvalidData => {
                        eprintln!("{}", make_utf8_err_str(&path));
                    }
                    _ => panic!("{}", e),
                },
            }

            (contents, path)
        }
    }
}

pub fn search_entry(entry: ArchiveEntry, text: &str) {
    // Extract text of entry
    let (contents, path) = match entry {
        ArchiveEntry::Bz2Entry(e) => extract_entry_text_and_path(ArchiveEntry::Bz2Entry(e)),
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
