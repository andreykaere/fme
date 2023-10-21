use audiotags::{MimeType, Picture, Tag};
use clap::Parser;
use id3::Version;

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;

use atty::Stream;

// TODO: create the ability to choose different modes
// TODO: think about how it's better to deal with non-ascii case
// around this

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(flatten)]
    metadata: Metadata,

    #[arg(long)]
    title_exec: Option<String>,

    #[arg(long)]
    artist_exec: Option<String>,

    files: Option<Vec<Box<Path>>>,
}

#[derive(clap::Args, Clone, Default, Debug)]
struct Metadata {
    #[arg(long, short)]
    title: Option<String>,

    #[arg(long, short)]
    artist: Option<String>,

    #[arg(short = 'T', long)]
    album_title: Option<String>,

    #[arg(short = 'C', long)]
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

fn init_metadata(file: &File) {
    match file
        .path
        .extension()
        .unwrap()
        .to_string_lossy()
        .to_string()
        .to_lowercase()
        .as_str()
    {
        "mp3" => {
            let new_tag = id3::Tag::new();
            new_tag
                .write_to_path(
                    file.path.to_string_lossy().to_string(),
                    Version::Id3v24,
                )
                .unwrap();
        }

        "m4a" | "m4b" | "m4p" | "m4v" | "isom" | "mp4" => {
            let new_tag = mp4ameta::Tag::default();
            new_tag
                .write_to_path(file.path.to_string_lossy().to_string())
                .unwrap();
        }

        "flac" => {
            let mut new_tag = metaflac::Tag::new();
            new_tag
                .write_to_path(file.path.to_string_lossy().to_string())
                .unwrap();
        }

        _ => unimplemented!("Other file formats are not supported"),
    }
}

fn write_metadata(file: &File, metadata: &Metadata) {
    let tag = Tag::new();
    let mut tag = match tag.read_from_path(&file.path) {
        Ok(t) => t,
        Err(_) => {
            init_metadata(file);
            tag.read_from_path(&file.path).unwrap()
        }
    };

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

    tag.write_to_path(&file.path.to_string_lossy()).unwrap();
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
        .to_string_lossy()
        .to_string()
}

fn artist_and_title(filename: &str) -> (String, String) {
    let (artist, title) = match filename.split_once('-') {
        Some((a, t)) => (a, t),
        None => ("", filename),
    };
    // .expect("Can't extract artist and title from the filename");

    let title = Path::new(title).file_stem().unwrap().to_string_lossy();

    let trim_num = |s: &str| -> String {
        let s_comps = s.split_once(' ');
        if let Some((num, s_pure)) = s_comps {
            if num.chars().all(|e| e.is_numeric()) {
                return s_pure.to_string();
            }
        }

        s.to_string()
    };

    // Trim leading number, because we don't want the number of the file to be
    // in the metadata
    let title = trim_num(title.trim());
    let artist = trim_num(artist.trim());

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

fn get_all_files(
    files_from_args: &[Box<Path>],
    files_from_stdin: &[Box<Path>],
) -> Vec<File> {
    let mut files = Vec::new();
    let files_iter = files_from_stdin.iter().chain(files_from_args);

    for file in files_iter {
        files.push(File::new(file));
    }

    files
}

fn main() {
    let args = Args::parse();

    let files_from_args = &args.files.unwrap_or(Vec::new());

    let files_from_stdin = if atty::is(Stream::Stdin) {
        Vec::new()
    } else {
        io::stdin()
            .lines()
            .map(|x| Path::new(&x.unwrap()).into())
            .collect()
    };

    let files = get_all_files(files_from_args, &files_from_stdin);

    files
        .iter()
        .for_each(|file| process_file(file, &args.metadata));
}
