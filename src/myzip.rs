use std::io::{Read, Seek};
use zip::read::ZipArchive;

use crate::archivetypes::ArchiveEntry;
use crate::search::search_entry;

pub fn unpack_and_search_zip<R>(mut archive: ZipArchive<R>, text: &str, _path: &str)
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
