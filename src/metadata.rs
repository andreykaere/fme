use anyhow::bail;
use audiotags::{MimeType, Picture, Tag};
use id3::Version;

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use crate::parse::ParsePattern;
use crate::Mode;

#[derive(clap::Args, Clone, Default, Debug)]
pub struct Metadata {
    #[arg(long, short)]
    pub title: Option<String>,

    #[arg(long, short)]
    pub artist: Option<String>,

    #[arg(long, visible_alias = "at")]
    pub album_title: Option<String>,

    #[arg(long, visible_alias = "ac")]
    pub album_cover: Option<PathBuf>,

    #[arg(long, short)]
    pub year: Option<u32>,

    #[arg(long, visible_alias = "tn")]
    pub track_number: Option<u16>,
}

impl Metadata {
    fn update(&mut self, metadata: &Metadata) {
        if metadata.artist.is_some() {
            self.artist = metadata.artist.clone();
        }

        if metadata.title.is_some() {
            self.title = metadata.title.clone();
        }

        if metadata.album_cover.is_some() {
            self.album_cover = metadata.album_cover.clone();
        }

        if metadata.album_title.is_some() {
            self.album_title = metadata.album_title.clone();
        }

        if metadata.year.is_some() {
            self.year = metadata.year;
        }

        if metadata.track_number.is_some() {
            self.track_number = metadata.track_number;
        }
    }
}

#[derive(Debug)]
pub struct AudioFile {
    path: PathBuf,
}

impl AudioFile {
    pub fn new(file: impl AsRef<Path>) -> Self {
        let ext = match file.as_ref().extension() {
            Some(x) => x.to_string_lossy().to_string().to_lowercase(),
            None => "".to_string(),
        };

        let path = file.as_ref().to_owned();

        if ext.is_empty() {
            eprintln!(
                "Can't figure out filetype of the file without extension"
            );
            std::process::exit(2);
        }

        if !is_supported_type(&ext) {
            eprintln!("Filetype '{ext}' is not supported");
            std::process::exit(2);
        }

        Self { path }
    }

    fn init_metadata(&self) {
        match self
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
                        self.path.to_string_lossy().to_string(),
                        Version::Id3v24,
                    )
                    .unwrap();
            }

            "wav" => {
                let new_tag = id3::Tag::new();
                new_tag
                    .write_to_wav_path(
                        self.path.to_string_lossy().to_string(),
                        Version::Id3v24,
                    )
                    .unwrap();
            }

            "m4a" | "m4b" | "m4p" | "m4v" | "isom" | "mp4" => {
                let new_tag = mp4ameta::Tag::default();
                new_tag
                    .write_to_path(self.path.to_string_lossy().to_string())
                    .unwrap();
            }

            "flac" => {
                let mut new_tag = metaflac::Tag::new();
                new_tag
                    .write_to_path(self.path.to_string_lossy().to_string())
                    .unwrap();
            }

            _ => unimplemented!("Other file formats are not supported"),
        }
    }

    pub fn write_metadata(&self, metadata: &Metadata) {
        let tag = Tag::new();
        let mut tag = match tag.read_from_path(&self.path) {
            Ok(t) => t,
            Err(_) => {
                self.init_metadata();

                // if self.path.extension().unwrap() == "wav" {
                //     // Id3v2Tag::read_from_wav_path(&self.path)
                //     //     .expect("Could not init metadata")
                // } else {
                tag.read_from_path(&self.path)
                    .expect("Could not init metadata")
                // }
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

        if let Some(track_number) = &metadata.track_number {
            tag.set_track_number(*track_number);
        }

        tag.write_to_path(&self.path.to_string_lossy()).unwrap();
    }

    pub fn process_file(
        &self,
        metadata: &Metadata,
        mode: Mode,
        parse_patterns: Option<&[ParsePattern]>,
    ) {
        let try_derive_metadata = match mode {
            Mode::FromFileName => {
                // TODO: THINK THROUGH UNWRAP
                self.metadata_from_filename(parse_patterns.unwrap())
            }

            Mode::FromInternet => self.metadata_from_internet(),
        };

        if let Ok(mut derived_metadata) = try_derive_metadata {
            derived_metadata.update(metadata);
            self.write_metadata(&derived_metadata);
        } else {
            eprintln!("Couldn't apply given patterns to the filename");
            std::process::exit(2);
        }
    }

    fn metadata_from_filename(
        &self,
        parse_patterns: &[ParsePattern],
    ) -> anyhow::Result<Metadata> {
        // println!("patterns: {:?}", parse_patterns);

        let filename =
            self.path.file_stem().unwrap().to_string_lossy().to_string();

        for pattern in parse_patterns {
            if let Ok(metadata) = pattern.try_pattern(&filename) {
                // println!(
                //     "WITH PATTERN: {:?}, PARSED METADATA: {:?}",
                //     pattern, metadata
                // );
                // println!("foo");
                return Ok(metadata);
            }
        }

        bail!("Failed to derive metadata from filename");
    }

    fn metadata_from_internet(&self) -> anyhow::Result<Metadata> {
        todo!();
    }
}

fn is_supported_type(ext: &str) -> bool {
    [
        "mp3", "wav", "flac", "mp4", "m4a", "m4b", "m4p", "m4v", "isom",
    ]
    .contains(&ext)
}
