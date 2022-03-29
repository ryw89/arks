use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use std::fs::File;
use tar::Entry;
use zip::read::ZipFile;

pub enum ArchiveEntry<'a> {
    Bz2Entry(Box<Entry<'a, BzDecoder<File>>>),
    GzEntry(Box<Entry<'a, GzDecoder<File>>>),
    ZipFile(Box<ZipFile<'a>>),
}

pub enum MimeType {
    Bz2,
    Gz,
    Zip,
}
