use audiotags::{MimeType, Picture, Tag};
use clap::Parser;

use std::ffi::OsStr;
use std::fs;
use std::io::{self, Lines, StdinLock};
use std::path::Path;

// TODO: create the ability to choose different modes
// TODO: think about how it's better to deal with non-ascii case
// TODO: allow stdin only from piping

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(flatten)]
    metadata: Metadata,

    title_exec: Option<String>,
    artist_exec: Option<String>,
    files: Option<Vec<String>>,
}

#[derive(clap::Args, Clone, Default, Debug)]
struct Metadata {
    #[arg(long, short)]
    title: Option<String>,

    #[arg(long, short)]
    artist: Option<String>,

    #[arg(long)]
    album_title: Option<String>,

    #[arg(long)]
    album_cover: Option<Box<Path>>,

    #[arg(long, short)]
    year: Option<u32>,
}

#[derive(Debug)]
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

fn write_metadata(file: &File, metadata: &Metadata) {
    let mut tag = Tag::new().read_from_path(&file.path).unwrap();

    if let Some(artist) = &metadata.artist {
        tag.set_artist(artist);
    }

    if let Some(title) = &metadata.title {
        tag.set_title(title);
    }

    if let Some(album_cover) = &metadata.album_cover {
        let cover = fs::read(album_cover).unwrap();
        let ext = album_cover.extension().and_then(OsStr::to_str).unwrap();
        let mimetype = match ext {
            "png" | "PNG" => MimeType::Png,
            "jpg" | "jpeg" | "JPG" => MimeType::Jpeg,
            _ => unimplemented!(),
        };

        let picture = Picture::new(&cover, mimetype);

        tag.set_album_cover(picture);
    }

    if let Some(album_title) = &metadata.album_title {
        tag.set_album_title(album_title);
    }

    if let Some(year) = &metadata.year {
        tag.set_year(*year as i32);
    }

    tag.write_to_path(file.path.to_str().unwrap()).unwrap();

    //     let mut opened_file =
    //         fs::File::create(&*file.path).expect("Failed to open the file");

    //     tag.write_to(&mut opened_file)
    //         .expect("Filed to change file's metadata");
}

fn process_file(file: &File, metadata: &Metadata) {
    let metadata_default = &file.metadata;

    write_metadata(file, metadata_default);
    write_metadata(file, metadata);
}

fn get_filename(file: impl AsRef<Path>) -> String {
    file.as_ref()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

fn artist_and_title(filename: &str) -> (String, String) {
    let (mut artist, mut title) = filename
        .split_once('-')
        .expect("Can't extract artist and title from the filename");

    title = Path::new(title).file_stem().unwrap().to_str().unwrap();

    let trim_num = |s: &str| -> String {
        let s_comps = s.split_once(' ');
        if let Some((num, s_pure)) = s_comps {
            if num.chars().all(|e| e.is_numeric()) {
                return s_pure.to_string();
            }
        }

        s.to_string()
    };

    title = title.trim();
    artist = artist.trim();

    // Trim leading number, because we don't want the number of the file to be
    // in the metadata
    let title = trim_num(title);
    let artist = trim_num(artist);

    (title, artist)
}

fn get_metadata(file: impl AsRef<Path>) -> Metadata {
    let mut metadata = Metadata::default();
    let filename = get_filename(file);
    let (title, artist) = artist_and_title(&filename);

    metadata.title = Some(title.to_string());
    metadata.artist = Some(artist.to_string());

    metadata
}

fn get_files_list(
    files_from_args: Vec<String>,
    files_from_stdin: Lines<StdinLock<'_>>,
) -> Vec<File> {
    let files_iter = files_from_args
        .into_iter()
        .chain(files_from_stdin.into_iter().map(|x| x.unwrap()));
    // .filter(|x| !x.is_empty());

    let mut files = Vec::new();

    for file in files_iter {
        files.push(File::new(&file));
    }

    files
}

fn main() {
    let args = Args::parse();
    let files_from_stdin = io::stdin().lines();
    let files_from_args = args.files.unwrap_or(Vec::new());
    let files = get_files_list(files_from_args, files_from_stdin);

    files
        .iter()
        .for_each(|file| process_file(file, &args.metadata));
}
