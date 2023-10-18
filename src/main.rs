use audiotags::Tag;
use std::io;
use std::path::Path;

fn process_file(filename: &str) {
    let mut tag = Tag::new().read_from_path(filename).unwrap();

    let (artist, mut title) = filename
        .split_once("-")
        .expect("Can't extract artist and title from the filename");

    title = Path::new(title).file_stem().unwrap().to_str().unwrap();

    println!("artist: {artist}, title: {title}");

    tag.set_artist(artist);
    tag.set_title(title);

    tag.write_to_path(filename).expect("Faile to save");
}

fn main() {
    let files = io::stdin().lines();

    for file in files {
        let filename = file.unwrap();

        if !filename.is_empty() {
            println!(r"{}", filename);
            process_file(&filename);
        } else {
            break;
        }
    }
}
