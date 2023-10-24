use audiotags::{MimeType, Picture, Tag};
use clap::Parser;
use id3::Version;

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

use atty::Stream;

// TODO: create the ability to choose different modes
// TODO: think about how it's better to deal with non-ascii case
// around this

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[clap(flatten)]
    metadata: Metadata,

    #[arg(long, visible_alias = "te")]
    title_exec: Option<String>,

    #[arg(long, visible_alias = "ae")]
    artist_exec: Option<String>,

    files: Vec<PathBuf>,
}

#[derive(clap::Args, Clone, Default, Debug)]
struct Metadata {
    #[arg(long, short)]
    title: Option<String>,

    #[arg(long, short)]
    artist: Option<String>,

    #[arg(long, visible_alias = "at")]
    album_title: Option<String>,

    #[arg(long, visible_alias = "ac")]
    album_cover: Option<PathBuf>,

    #[arg(long, short)]
    year: Option<u32>,
}

#[derive(Debug)]
struct File {
    path: PathBuf,
    metadata: Metadata,
}

fn get_extension(file: impl AsRef<Path>) -> String {
    let ext = file.as_ref().extension();

    match ext {
        Some(x) => x.to_string_lossy().to_string().to_lowercase(),
        None => "".to_string(),
    }
}

fn is_supported_type(ext: &str) -> bool {
    ["mp3", "flac", "mp4", "m4a", "m4b", "m4p", "m4v", "isom"].contains(&ext)
}

impl File {
    fn new(file: impl AsRef<Path>) -> Self {
        let ext = get_extension(&file);

        if ext.is_empty() {
            println!("Can't figure out filetype of the file without extension");
            process::exit(2);
        }

        if !is_supported_type(&ext) {
            println!("Filetype '{ext}' is not supported");
            process::exit(2);
        }

        let path = file.as_ref().to_owned();
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
            tag.read_from_path(&file.path)
                .expect("Could not init metadata")
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

fn process_file(file: &File, metadata_specified: &Metadata) {
    let metadata = &file.metadata;

    write_metadata(file, metadata);
    write_metadata(file, metadata_specified);
}

fn get_filename(file: impl AsRef<Path>) -> String {
    file.as_ref()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string()
}

fn artist_and_title(filename: &str) -> (String, String) {
    let (artist, title) = if let Some((a, t)) = filename.split_once('-') {
        (a, t)
    } else {
        match filename.split_once('–') {
            Some((a, t)) => (a, t),
            None => ("", filename),
        }
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
    files_from_args: &[PathBuf],
    files_from_stdin: &[PathBuf],
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

    let files_from_args = &args.files;

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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory as _;
    use clap_complete::{generate, Generator, Shell};
    use std::fs;

    #[test]
    fn generate_completions() {
        let mut cmd = Args::command();

        for (shell, file) in &[
            (Shell::Bash, "fme.bash"),
            (Shell::Fish, "fme.fish"),
            (Shell::Zsh, "_fme"),
        ] {
            let mut file =
                fs::File::create(format!("./extra/completions/{}", file))
                    .unwrap();

            clap_complete::generate(*shell, &mut cmd, "fme", &mut file);
        }
    }
}
