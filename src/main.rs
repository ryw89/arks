#![feature(mutex_unlock)]

mod archivetypes;
mod file_types;
mod macros;
mod mytar;
mod myzip;
#[cfg(target_family = "unix")]
mod pipe;
mod search;

use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::ErrorKind;
use std::process;
use std::sync::Mutex;
use structopt::StructOpt;
use tar::Archive;
use zip::read::ZipArchive;

#[macro_use]
extern crate lazy_static;

use crate::archivetypes::MimeType;
use crate::mytar::*;
use crate::myzip::*;
#[cfg(target_family = "unix")]
use crate::pipe::reset_signal_pipe_handler;

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

    /// Match on within-archive file names
    #[structopt(short = "n", long = "inner", name = "INNER FILE PATTERN")]
    inner: Option<String>,
}

struct Config {
    inner_pattern: Mutex<Option<String>>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            inner_pattern: Mutex::new(None),
        }
    }
}

lazy_static! {
    static ref CONFIG: Config = Config {
        ..Default::default()
    };
}

/// Verify the file as an archive
fn verify_as_archive(path: &str) -> Result<MimeType, String> {
    let err_msg = format!("{} does not appear to be a valid archive.", path);
    match mime_guess::from_path(path).first_raw() {
        None => Err(err_msg),
        Some(g) => match g {
            "application/x-bzip2" => Ok(MimeType::Bz2),
            "application/x-gzip" => Ok(MimeType::Gz),
            "application/zip" => Ok(MimeType::Zip),
            _ => Err(err_msg),
        },
    }
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

fn main() {
    #[cfg(target_family = "unix")]
    reset_signal_pipe_handler().unwrap();
    let opt = Opt::from_args();

    // Write config to static
    let mut guard = CONFIG.inner_pattern.lock().unwrap();
    *guard = opt.inner.clone();
    Mutex::unlock(guard);

    // First, verify file exists & and is a .gz or .zip
    let file = load_file(&opt.file).unwrap_or_else(|e| bad_exit!(&e));
    let mime_type = verify_as_archive(&opt.file).unwrap_or_else(|e| bad_exit!(&e));

    match mime_type {
        MimeType::Bz2 => {
            let tar = BzDecoder::new(file);
            let archive = Archive::new(tar);
            unpack_and_search_tar(archive, &opt.text, &opt.file);
        }
        MimeType::Gz => {
            let tar = GzDecoder::new(file);
            let archive = Archive::new(tar);
            unpack_and_search_tar(archive, &opt.text, &opt.file);
        }
        MimeType::Zip => {
            let archive = ZipArchive::new(file).unwrap();
            unpack_and_search_zip(archive, &opt.text, &opt.file);
        }
    }
}
