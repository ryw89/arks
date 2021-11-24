mod archivetypes;
mod file_types;
mod macros;
mod myzip;
mod search;
mod targz;

use flate2::read::GzDecoder;
use std::fs::File;
use std::io::ErrorKind;
use std::process;
use structopt::StructOpt;
use tar::Archive;
use zip::read::ZipArchive;

use crate::archivetypes::MimeType;
use crate::myzip::*;
use crate::targz::*;

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
fn verify_as_archive(path: &str) -> Result<MimeType, String> {
    let guess = mime_guess::from_path(path).first_raw().unwrap();
    match guess {
        "application/x-gzip" => Ok(MimeType::Gz),
        "application/zip" => Ok(MimeType::Zip),
        _ => Err(format!("{} does not appear to be a valid archive.", path)),
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
    let opt = Opt::from_args();

    // First, verify file exists & and is a .gz or .zip
    let file = load_file(&opt.file).unwrap_or_else(|e| bad_exit!(&e));
    let mime_type = verify_as_archive(&opt.file).unwrap_or_else(|e| bad_exit!(&e));

    match mime_type {
        MimeType::Gz => {
            let tar = GzDecoder::new(file);
            let archive = Archive::new(tar);
            unpack_and_search_targz(archive, &opt.text, &opt.file);
        }
        MimeType::Zip => {
            let archive = ZipArchive::new(file).unwrap();
            unpack_and_search_zip(archive, &opt.text, &opt.file);
        }
    }
}
