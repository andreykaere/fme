use audiotags::Tag;
use std::env;
use std::fs;
use std::io;
use std::path::Path;

// TODO: create the ability to choose different modes
// TODO: think about how it's better to deal with non-ascii case

#[derive(Clone, Default)]
struct Metadata {
    title: Option<String>,
    artist: Option<String>,
    album_title: Option<String>,
    album_cover: Option<Box<Path>>,
    year: Option<u32>,
}

struct File {
    path: Box<Path>,
    metadata: Metadata,
}

impl File {
    fn new(file: impl AsRef<Path>) -> Self {
        let path = file.as_ref().to_owned().into();
        let metadata = get_metadata(&file);

        Self { path, metadata }
    }
}

fn write_metadata(file: &File) {
    let mut tag = Tag::new().read_from_path(&*file.path).unwrap();
    let metadata = &file.metadata;

    if let Some(artist) = &metadata.artist {
        tag.set_artist(artist);
    }

    if let Some(title) = &metadata.title {
        tag.set_title(title);
    }

    if let Some(album_title) = &metadata.album_title {
        tag.set_album_title(album_title);
    }

    if let Some(year) = &metadata.year {
        tag.set_year(*year as i32);
    }

    let mut opened_file =
        fs::File::create(&*file.path).expect("Failed to open the file");

    tag.write_to(&mut opened_file)
        .expect("Filed to change file's metadata");
}

fn get_filename(file: impl AsRef<Path>) -> String {
    file.as_ref()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

fn get_metadata(file: impl AsRef<Path>) -> Metadata {
    let mut metadata = Metadata::default();
    let filename = get_filename(file);

    let (artist, mut title) = filename
        .split_once("-")
        .expect("Can't extract artist and title from the filename");

    title = Path::new(title).file_stem().unwrap().to_str().unwrap();

    metadata.title = Some(title.to_string());
    metadata.artist = Some(artist.to_string());

    metadata
}

fn get_files_from_args() -> Vec<String> {
    todo!();
}

fn get_files_list() -> Vec<File> {
    let files_from_stdin = io::stdin().lines();
    let files_from_args = get_files_from_args();

    let files_iter = files_from_args
        .into_iter()
        .chain(files_from_stdin.into_iter().map(|x| x.unwrap()));

    let mut files = Vec::new();

    for file in files_iter {
        files.push(File::new(&file));
    }

    files
}

fn main() {
    let files = get_files_list();

    files.iter().for_each(|file| write_metadata(&file));
}
