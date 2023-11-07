use atty::Stream;
use clap::Parser;
use std::io;
use std::path::{Path, PathBuf};

mod metadata;
mod parse;

use metadata::{AudioFile, Metadata};
use parse::ParsePattern;

// TODO: think about how it's better to deal with non-ascii case
// around this

// TODO: think about extracting existing metadata and having an option of
// overwriting existing metadata or not. It maybe useful, when existing
// metadata is right and the automatically derived one is wrong.
//

// TODO: drop `audiotags` in favor of `lofty`

#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
pub struct Opts {
    #[clap(flatten)]
    metadata: Metadata,

    #[arg(
        long,
        short,
        value_enum,
        default_value_t = Mode::FromFileName,
        help = "Set the mode that will be used by the program to determine metadata",
    )]
    mode: Mode,

    /// When this option is specified, the program will try to match the given
    /// patterns (in the given order) with the filename and write extracted
    /// information to the metadata.
    ///
    /// You can use the following correspondence when writing your pattern:
    /// {n}    Artist <-> {a}
    /// {n}    Title  <-> {t}
    /// {n}    Album  <-> {m}
    /// {n}    Year   <-> {y}
    /// {n}    Track  <-> {d}
    ///
    /// When no parser pattern specified parser tries to apply the following
    /// default patters in the given order:
    /// {n}   1. {d} {a} - {t}
    /// {n}   2. {d} {a} — {t}
    /// {n}   3. {d}. {a} - {t}
    /// {n}   4. {d}. {a} — {t}
    /// {n}   5. {a} - {d} {t}
    /// {n}   6. {a} — {d} {t}
    /// {n}   7. {a} - {d}. {t}
    /// {n}   8. {a} — {d}. {t}
    /// {n}   9. {a} - {t}
    /// {n}  10. {a} — {t}
    /// {n}  11. {d} {t}
    /// {n}  12. {d}. {t}
    /// {n}  13. {t}
    #[arg(
        long,
        short,
        help = "Derive metadata information from the filename using specified patterns"
    )]
    parse: Option<Vec<ParsePattern>>,

    files: Vec<PathBuf>,
}

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
pub enum Mode {
    #[value(alias = "f", name = "filename")]
    FromFileName,

    #[value(alias = "i", name = "internet")]
    FromInternet,
}

fn get_all_files(
    files_from_args: &[PathBuf],
    files_from_stdin: &[PathBuf],
) -> Vec<AudioFile> {
    let mut files = Vec::new();
    let files_iter = files_from_stdin.iter().chain(files_from_args);

    for file in files_iter {
        match AudioFile::new(file) {
            Ok(audio_file) => files.push(audio_file),
            Err(e) => eprintln!("{e}"),
        }
    }

    files
}

fn main() {
    let args = Opts::parse();
    let files_from_args = &args.files;
    let metadata = &args.metadata;
    let mode = args.mode;
    let parse_patterns =
        &args.parse.unwrap_or(ParsePattern::default_patterns());

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
        .for_each(|file| file.process_file(metadata, mode, parse_patterns));
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use clap::CommandFactory as _;
//     use clap_complete::{generate, Generator, Shell};
//     use std::fs;

//     #[test]
//     fn generate_completions() {
//         let mut cmd = Opts::command();

//         for (shell, file) in &[
//             (Shell::Bash, "fme.bash"),
//             (Shell::Fish, "fme.fish"),
//             (Shell::Zsh, "_fme"),
//         ] {
//             let mut file =
//                 fs::File::create(format!("./extra/completions/{}", file))
//                     .unwrap();

//             clap_complete::generate(*shell, &mut cmd, "fme", &mut file);
//         }
//     }
// }
