use colored::*;
use flate2::read::GzDecoder;
use std::fs;
use std::fs::{read_to_string, File};
use std::hint::unreachable_unchecked;
use std::io::{self, ErrorKind, Read, Seek, Write};
use std::process;
use structopt::StructOpt;
use tar::{Archive, Entry};
use tempfile::NamedTempFile;
use zip::read::{ZipArchive, ZipFile};

/// Search for text within archives
#[derive(StructOpt, Debug)]
#[structopt(name = "arks")]
struct Opt {
    /// Text to search for
    #[structopt(name = "TEXT")]
    text: String,

    /// Archive to search
    #[structopt(name = "FILE")]
    file: String,
}

/// Verify the file as an archive
fn verify_as_archive(path: &str) -> Result<String, String> {
    let guess = mime_guess::from_path(path).first_raw().unwrap();
    let allowed = vec!["application/x-gzip", "application/zip"];
    if !allowed.iter().any(|&i| i == guess) {
        return Err(format!("{} does not appear to be a valid archive.", path));
    }
    Ok(guess.to_string())
}

/// Prints an error message and exits the program.
macro_rules! bad_exit {
    ( $e:expr ) => {{
        eprintln!("{}", $e);
        process::exit(1);
    }};
}

/// Open a file given a path
fn load_file(path: &str) -> Result<File, String> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                return Err(format!("File not found: {}.", path));
            }
            _ => return Err(e.to_string()),
        },
    };
    Ok(file)
}

enum ArchiveEntry<'a> {
    GzEntry(Entry<'a, GzDecoder<File>>),
    ZipFile(ZipFile<'a>),
}

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

            return (contents, path);
        }
        ArchiveEntry::ZipFile(mut e) => {
            let path = e.name().to_string();
            let mut contents = String::new();
            e.read_to_string(&mut contents).unwrap();

            return (contents, path);
        }
    }
}

fn search_entry(entry: ArchiveEntry, text: &str) {
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
            print!("\n");
            io::stdout().flush().unwrap();
        }
    });
}

fn unpack_and_search_gz_entry(entry: Entry<GzDecoder<File>>, text: &str) {
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

fn unpack_and_search_targz(mut archive: Archive<GzDecoder<File>>, text: &str, path: &str) {
    archive
        .entries()
        .unwrap_or_else(|e| bad_exit!(&e))
        .for_each(|x| match x {
            Ok(e) => unpack_and_search_gz_entry(e, text),
            Err(_) => {
                eprintln!("Warning: Could not extract an entry in {}", path.red());
            }
        });
}

fn unpack_and_search_zip<R>(mut archive: ZipArchive<R>, text: &str, _path: &str)
where
    R: Seek,
    R: Read,
{
    let mut names = Vec::new();
    for i in 0..archive.len() {
        names.push(archive.by_index(i).unwrap().name().to_string());
    }

    for name in names.iter() {
        match mime_guess::from_path(name).first_raw() {
            None => continue,
            Some(m) => {
                if m.starts_with("text/") {
                    let file = archive.by_name(name).unwrap();
                    search_entry(ArchiveEntry::ZipFile(file), text);
                }
            }
        }
    }
}

fn main() {
    let opt = Opt::from_args();

    // First, verify file exists & and is a .gz or .zip
    let file = load_file(&opt.file).unwrap_or_else(|e| bad_exit!(&e));
    let mime_type = verify_as_archive(&opt.file).unwrap_or_else(|e| bad_exit!(&e));

    match mime_type.as_ref() {
        "application/x-gzip" => {
            let tar = GzDecoder::new(file);
            let archive = Archive::new(tar);
            unpack_and_search_targz(archive, &opt.text, &opt.file);
        }
        "application/zip" => {
            let archive = ZipArchive::new(file).unwrap();
            unpack_and_search_zip(archive, &opt.text, &opt.file);
        }
        _ => unsafe {
            // Will never hit hit this branch due to the check in
            // verify_as_archive().
            unreachable_unchecked();
        },
    }
}
