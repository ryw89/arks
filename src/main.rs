use flate2::read::GzDecoder;
use std::fs::File;
use std::io::ErrorKind;
use std::process;
use structopt::StructOpt;
use tar::Archive;

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
fn verify_mime_type(path: &str) -> Result<(), String> {
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

fn main() {
    let opt = Opt::from_args();
    let file = load_file(&opt.file).unwrap_or_else(|e| bad_exit!(&e));
    verify_mime_type(&opt.file).unwrap_or_else(|e| bad_exit!(&e));

    let tar = GzDecoder::new(file);
    let mut archive = Archive::new(tar);
}
