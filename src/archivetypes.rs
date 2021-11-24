use flate2::read::GzDecoder;
use std::fs::File;
use tar::Entry;
use zip::read::ZipFile;

pub enum ArchiveEntry<'a> {
    GzEntry(Entry<'a, GzDecoder<File>>),
    ZipFile(ZipFile<'a>),
}

pub enum MimeType {
    Gz,
    Zip,
}
