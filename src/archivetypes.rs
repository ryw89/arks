use flate2::read::GzDecoder;
use std::fs::File;
use tar::Entry;
use zip::read::ZipFile;

pub enum ArchiveEntry<'a> {
    GzEntry(Box<Entry<'a, GzDecoder<File>>>),
    ZipFile(Box<ZipFile<'a>>),
}

pub enum MimeType {
    Gz,
    Zip,
}
