use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use audiotags::{MimeType, Picture, Tag};
use id3::Version;
use regex::Regex;

use crate::parse::ParsePattern;
use crate::{FilenameMode, Mode};

#[derive(clap::Args, Clone, Default, Debug)]
pub struct Metadata {
    /// Write specified value to the 'title' tag
    #[arg(long, short)]
    pub title: Option<String>,

    /// Write specified value to the 'artist' tag
    #[arg(long, short)]
    pub artist: Option<String>,

    /// Write specified value to the 'album' tag
    #[arg(long, visible_alias = "at")]
    pub album_title: Option<String>,

    /// Set the image, located at the given path, as an album cover
    #[arg(long, visible_alias = "ac")]
    pub album_cover: Option<PathBuf>,

    /// Write specified value to the 'year' tag
    #[arg(long, short)]
    pub year: Option<u32>,

    /// Write specified value to the 'track number' tag
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
    pub fn new(file: impl AsRef<Path>) -> anyhow::Result<Self> {
        let ext = match file.as_ref().extension() {
            Some(x) => x.to_string_lossy().to_string().to_lowercase(),
            None => "".to_string(),
        };

        let path = file.as_ref().to_owned();

        if !path.is_file() {
            bail!("There is no such file: '{}'", path.to_string_lossy());
        }

        if ext.is_empty() {
            bail!(
                "Can't figure out filetype of the file '{}', \
            because there is no extension",
                path.to_string_lossy()
            );
        }

        if !is_supported_type(&ext) {
            bail!("Filetype '{ext}' is not supported");
        }

        Ok(Self { path })
    }

    fn init_metadata(&self) -> anyhow::Result<()> {
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
                new_tag.write_to_path(
                    self.path.to_string_lossy().to_string(),
                    Version::Id3v24,
                )?
            }

            "wav" => {
                let new_tag = id3::Tag::new();
                new_tag.write_to_wav_path(
                    self.path.to_string_lossy().to_string(),
                    Version::Id3v24,
                )?
            }

            "m4a" | "m4b" | "m4p" | "m4v" | "isom" | "mp4" => {
                let new_tag = mp4ameta::Tag::default();
                new_tag
                    .write_to_path(self.path.to_string_lossy().to_string())?
            }

            "flac" => {
                let mut new_tag = metaflac::Tag::new();
                new_tag
                    .write_to_path(self.path.to_string_lossy().to_string())?
            }

            _ => unimplemented!("Other file formats are not supported"),
        }

        Ok(())
    }

    pub fn write_metadata(&self, metadata: &Metadata) -> anyhow::Result<()> {
        let tag = Tag::new();
        let mut tag = match tag.read_from_path(&self.path) {
            Ok(t) => t,
            Err(_) => {
                if self.init_metadata().is_err() {
                    bail!(
                        "Failed to init metadata tags in the file '{}'",
                        self.path.file_name().unwrap().to_string_lossy()
                    );
                }

                match tag.read_from_path(&self.path) {
                    Ok(t) => t,
                    Err(_) => bail!(
                        "Failed to read metadata tags from the file '{}'",
                        self.path.file_name().unwrap().to_string_lossy()
                    ),
                }
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
                _ => unimplemented!(
                    "Other image formats are not supported for album cover"
                ),
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

        if tag.write_to_path(&self.path.to_string_lossy()).is_err() {
            bail!(
                "Failed to write metadata tags in the file '{}'",
                self.path.file_name().unwrap().to_string_lossy()
            );
        }

        Ok(())
    }

    pub fn process_file(
        &self,
        metadata: &Metadata,
        mode: Mode,
        filename_mode: &FilenameMode,
    ) {
        let try_derive_metadata = match mode {
            Mode::FromFilename => match filename_mode {
                FilenameMode::Parse(parse_patterns) => {
                    self.parse_metadata_from_filename(parse_patterns)
                }

                FilenameMode::Regex(regex) => {
                    self.regex_metadata_from_filename(regex, metadata)
                }
            },

            Mode::FromInternet => self.metadata_from_internet(),
        };

        if let Ok(mut derived_metadata) = try_derive_metadata {
            match filename_mode {
                // We don't want to write specified metadata in case of regex,
                // because it has been already written with needed tokens applied
                FilenameMode::Regex(_) => {}

                _ => derived_metadata.update(metadata),
            }

            if let Err(e) = self.write_metadata(&derived_metadata) {
                eprintln!("{e}");
            }
        } else {
            eprintln!(
                "Couldn't apply given patterns to the filename '{}'",
                self.path.file_stem().unwrap().to_string_lossy()
            );
        }
    }

    fn filename(&self) -> String {
        self.path.file_stem().unwrap().to_string_lossy().to_string()
    }

    fn regex_metadata_from_filename(
        &self,
        regex: &str,
        metadata: &Metadata,
    ) -> anyhow::Result<Metadata> {
        let filename = self.filename();
        let mut metadata = metadata.clone();

        let re = Regex::new(regex)?;
        let captures = re
            .captures(&filename)
            .context("Couldn't apply regex to this filename: {filename}")?;

        for i in 1..captures.len() {
            let token = format!("${{{i}}}");
            let replace_token = &captures[i];

            println!("token: {token}, replace: {:?}", replace_token);

            if let Some(artist) = metadata.artist {
                metadata.artist = Some(artist.replace(&token, replace_token));
            }

            if let Some(title) = metadata.title {
                metadata.title = Some(title.replace(&token, replace_token));
            }

            if let Some(track_number) = metadata.track_number {
                let track_number = track_number
                    .to_string()
                    .replace(&token, replace_token)
                    .parse()
                    .context(
                        "You can only put a number in tag 'track_number'",
                    )?;

                metadata.track_number = Some(track_number);
            }

            if let Some(album_title) = metadata.album_title {
                metadata.album_title =
                    Some(album_title.replace(&token, replace_token));
            }

            if let Some(year) = metadata.year {
                let year = year
                    .to_string()
                    .replace(&token, replace_token)
                    .parse()
                    .context("You can only put a number in tag 'year'")?;

                metadata.year = Some(year);
            }
        }

        Ok(metadata)
    }

    fn parse_metadata_from_filename(
        &self,
        parse_patterns: &[ParsePattern],
    ) -> anyhow::Result<Metadata> {
        let filename = self.filename();

        for pattern in parse_patterns {
            if let Ok(metadata) = pattern.try_pattern(&filename) {
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
