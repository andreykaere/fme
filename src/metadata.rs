use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use audiotags::{MimeType, Picture, Tag};
use id3::Version;
use regex::Regex;

use crate::parse::ParsePattern;
use crate::{FilenameParseMode, Mode};

#[derive(clap::Args, Clone, Default, Debug)]
pub struct Metadata {
    /// Write specified value to the 'title' tag
    #[arg(long, short)]
    pub title: Option<String>,

    /// Write specified value to the 'artist' tag
    #[arg(long, short)]
    pub artist: Option<String>,

    /// Write specified value to the 'album' tag
    #[arg(long, short = 'm', visible_alias = "at")]
    pub album_title: Option<String>,

    /// Set the image, located at the given path, as an album cover
    #[arg(long, visible_alias = "ac")]
    pub album_cover: Option<PathBuf>,

    /// Write specified value to the 'year' tag
    #[arg(long, short)]
    pub year: Option<NumberOrToken>,

    /// Write specified value to the 'track number' tag
    #[arg(long, short = 'd', visible_alias = "tn")]
    pub track_number: Option<NumberOrToken>,
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
            self.year = metadata.year.clone();
        }

        if metadata.track_number.is_some() {
            self.track_number = metadata.track_number.clone();
        }
    }
}

#[derive(Debug, Clone)]
pub enum NumberOrToken {
    Number(u32),
    Token(String),
}

impl std::str::FromStr for NumberOrToken {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if string.chars().all(char::is_numeric) {
            Ok(Self::Number(string.parse()?))
        } else {
            Ok(Self::Token(string.to_string()))
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
            None => String::new(),
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

    fn path(&self) -> String {
        self.path.to_string_lossy().to_string()
    }

    fn filename_stem(&self) -> String {
        self.path.file_stem().unwrap().to_string_lossy().to_string()
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
                new_tag.write_to_path(self.path(), Version::Id3v24)?;
            }

            "wav" => {
                let new_tag = id3::Tag::new();
                new_tag.write_to_wav_path(self.path(), Version::Id3v24)?;
            }

            "m4a" | "m4b" | "m4p" | "m4v" | "isom" | "mp4" => {
                let new_tag = mp4ameta::Tag::default();
                new_tag.write_to_path(self.path())?;
            }

            "flac" => {
                let mut new_tag = metaflac::Tag::new();
                new_tag.write_to_path(self.path())?;
            }

            _ => unimplemented!("Other file formats are not supported"),
        }

        Ok(())
    }

    pub fn write_metadata(&self, metadata: &Metadata) -> anyhow::Result<()> {
        let tag = Tag::new();
        let filename = self.path.file_name().unwrap().to_string_lossy();

        let mut tag = if let Ok(tag_import) = tag.read_from_path(&self.path) {
            tag_import
        } else {
            self.init_metadata().context(format!(
                "Failed to init metadata tags in the file '{filename}'",
            ))?;

            tag.read_from_path(&self.path).context(format!(
                "Failed to read metadata tags from the file '{filename}'",
            ))?
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
            if let NumberOrToken::Number(n) = year {
                tag.set_year(*n as i32);
            } else {
                bail!(
                    "Can't write token to metadata tag 'year', \
                something went wrong in the program. Please, report a bug."
                );
            }
        }

        if let Some(track_number) = &metadata.track_number {
            if let NumberOrToken::Number(t) = track_number {
                tag.set_track_number(*t as u16);
            } else {
                bail!(
                    "Can't write token to metadata tag 'track_number', \
                something went wrong in the program. Please, report a bug."
                );
            }
        }


        tag.write_to_path(&self.path()).context(format!(
            "Failed to write metadata tags in the file '{filename}'",
        ))?;

        Ok(())
    }

    pub fn process_file(
        &self,
        metadata: &Metadata,
        mode: Mode,
        filename_parse_mode: &FilenameParseMode,
    ) -> anyhow::Result<()> {
        let try_derive_metadata;

        match mode {
            Mode::FromFilename => match filename_parse_mode {
                FilenameParseMode::Parser(parse_patterns) => {
                    match self.parse_metadata_from_filename(parse_patterns) {
                        Ok(x) => try_derive_metadata = Some(x),
                        Err(e) => bail!(
                            "Couldn't apply given patterns to the filename '{}', \
                            the following error occurred: {e}",
                            self.filename_stem()
                        ),
                    };
                }

                FilenameParseMode::Regex(regex) => {
                    match self.regex_metadata_from_filename(regex, metadata) {
                        Ok(x) => try_derive_metadata = Some(x),
                        Err(e) => bail!(
                            "Couldn't apply given regex to the filename '{}', \
                            the following error occurred: {e}",
                            self.filename_stem()
                        ),
                    };
                }
            },

            Mode::FromInternet => todo!(), //self.metadata_from_internet(),
        };

        if let Some(mut derived_metadata) = try_derive_metadata {
            match filename_parse_mode {
                // We don't want to write specified metadata in case of regex,
                // because it has been already written with needed tokens applied
                FilenameParseMode::Regex(_) => {}

                _ => derived_metadata.update(metadata),
            }

            self.write_metadata(&derived_metadata)?;
        }

        Ok(())
    }

    fn regex_metadata_from_filename(
        &self,
        regex: &str,
        metadata: &Metadata,
    ) -> anyhow::Result<Metadata> {
        let filename_stem = self.filename_stem();
        let mut metadata = metadata.clone();

        let re = Regex::new(regex)?;
        let captures = re.captures(&filename_stem).context(
            "Couldn't apply regex to this filename: {filename_stem}",
        )?;

        for i in 1..captures.len() {
            let token = format!("${{{i}}}");
            let replace_token = &captures[i];

            if let Some(artist) = metadata.artist {
                metadata.artist = Some(artist.replace(&token, replace_token));
            }

            if let Some(title) = metadata.title {
                metadata.title = Some(title.replace(&token, replace_token));
            }

            if let Some(NumberOrToken::Token(track_number)) =
                metadata.track_number
            {
                let track_number = track_number
                    .to_string()
                    .replace(&token, replace_token)
                    .parse()
                    .context(
                        "You can only put a number in tag 'track_number'",
                    )?;

                metadata.track_number =
                    Some(NumberOrToken::Number(track_number));
            }

            if let Some(album_title) = metadata.album_title {
                metadata.album_title =
                    Some(album_title.replace(&token, replace_token));
            }

            if let Some(NumberOrToken::Token(year)) = metadata.year {
                let year = year
                    .to_string()
                    .replace(&token, replace_token)
                    .parse()
                    .context("You can only put a number in tag 'year'")?;

                metadata.year = Some(NumberOrToken::Number(year));
            }
        }

        Ok(metadata)
    }

    fn parse_metadata_from_filename(
        &self,
        parse_patterns: &[ParsePattern],
    ) -> anyhow::Result<Metadata> {
        let filename_stem = self.filename_stem();

        for pattern in parse_patterns {
            if let Ok(metadata) = pattern.try_pattern(&filename_stem) {
                return Ok(metadata);
            }
        }

        bail!("Failed to derive metadata from this filename: {filename_stem}");
    }

    // fn metadata_from_internet(&self) -> anyhow::Result<Metadata> {
    //     todo!();
    // }
}

fn is_supported_type(ext: &str) -> bool {
    [
        "mp3", "wav", "flac", "mp4", "m4a", "m4b", "m4p", "m4v", "isom",
    ]
    .contains(&ext)
}
