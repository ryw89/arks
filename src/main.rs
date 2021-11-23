use colored::*;
use flate2::read::GzDecoder;
use std::fs::{read_to_string, File};
use std::io::{self, ErrorKind, Write};
use std::process;
use structopt::StructOpt;
use tar::{Archive, Entry};
use tempfile::NamedTempFile;

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
fn verify_as_archive(path: &str) -> Result<(), String> {
    let guess = mime_guess::from_path(path).first_raw().unwrap();
    let allowed = vec!["application/x-gzip"];
    if !allowed.iter().any(|&i| i == guess) {
        return Err(format!("{} does not appear to be a valid archive.", path));
    }
    Ok(())
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

fn search_entry(mut entry: Entry<GzDecoder<File>>, text: &str) {
    // Unpack entry to the filesystem
    let file = NamedTempFile::new().unwrap();
    entry.unpack(&file).unwrap();

    // Get entry path
    let path = entry.path().unwrap().display().to_string();

    // Read into string
    let contents = read_to_string(&file.path()).unwrap();

    contents.lines().enumerate().for_each(|(i, line)| {
        if line.contains(text) {
            print!("{}:{}:", path.red(), (i + 1).to_string().yellow());

            for s in line.split_inclusive(text) {
                if s.contains(text) {
                    let other_bit = s.split(text).collect::<Vec<&str>>()[0];

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

fn unpack_and_search(entry: Entry<GzDecoder<File>>, text: &str) {
    // Infer MIME type
    let path = entry.path().unwrap().display().to_string();
    match mime_guess::from_path(&path).first_raw() {
        None => {
            // Occurs for directories & extensionless files
        }
        Some(m) => {
            if m.starts_with("text/") {
                search_entry(entry, text);
            } else {
            }
        }
    }
}

fn main() {
    let opt = Opt::from_args();

    // First, verify file exists & and is a .gz file
    let file = load_file(&opt.file).unwrap_or_else(|e| bad_exit!(&e));
    verify_as_archive(&opt.file).unwrap_or_else(|e| bad_exit!(&e));

    let tar = GzDecoder::new(file);
    let mut archive = Archive::new(tar);

    archive
        .entries()
        .unwrap_or_else(|e| bad_exit!(&e))
        .for_each(|x| match x {
            Ok(e) => unpack_and_search(e, &opt.text),
            Err(_) => {
                eprintln!("Warning: Could not extract an entry in {}", &opt.file.red());
            }
        });
}
