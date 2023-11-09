This is `fme` — flexible metadata editor. You can supply files both as
arguments and as piping from other command like `find` or `ls`. 

If `fme` fails to get metadata for given file or fails to write extracted
metadata to it, `fme` will just print a error message and will continue to the
next file.

## Usage

### Options

```
Usage: fme [OPTIONS] [FILES]...

Arguments:
  [FILES]...

Options:
  -t, --title <TITLE>                Write specified value to the 'title' tag
  -a, --artist <ARTIST>              Write specified value to the 'artist' tag
      --album-title <ALBUM_TITLE>    Write specified value to the 'album' tag [aliases: at]
      --album-cover <ALBUM_COVER>    Set the image, located at the given path, as an album cover [aliases: ac]
  -y, --year <YEAR>                  Write specified value to the 'year' tag
      --track-number <TRACK_NUMBER>  Write specified value to the 'track number' tag [aliases: tn]
  -p, --parse <PARSE>                Derive metadata information from the filename using specified patterns
  -e, --regex <REGEX>                Try to apply regex to the filename and writes matched groups to special tokens
  -h, --help                         Print help (see more with '--help')
  -V, --version                      Print version
```

It is recommended to use [`rnr`](https://github.com/ismaelgv/rnr) utility to
rename files beforehand if you need it (it might be useful if you download it,
for example, from some youtube playlist and you want it to format first,
before using `--parse` option).


## Examples

### Basic examples

- Manually specification
```
fme "Foo - Bar.mp3" -a Foo -t Bar
```

- Parsing mode, using default parsers:
```
fme "Foo - Bar.mp3"
```
In this case, the result will be as in previous example. Here we didn't use
`--parse` option, because this pattern is one of the default ones and
therefore does not need to be specified manually. For the complete list of
default patterns see `fme --help` in the `--parse` option section.

- Unknown to the parser pattern. This time we have to specify it manually:
```
fme -p '{d}. {a} - {t} [{m}]' "12. Foo - Bar [Quuz].mp3"
```

- Regex
```
fme -e '(\d+)\. (\w+) - (\w+) \[(\w+)\]' --tn '${1}' -a '${2}' -t '${3}' --at '${4}' "12. Foo - Bar [Quuz].mp3"
```


### Advanced example

Here is represented one of the typical examples where `fme` comes in handy
along with other tools. Let's say that we want to turn this
[video](https://youtu.be/A4lTUgJWNIw) to an album of audio files with built-in
names of songs, artist year and album cover. Of course, doing it by hands
will take eternity! So, we start off with downloading it as audio file using
[`yt-dlp`](https://github.com/yt-dlp/yt-dlp):
```bash 
yt-dlp -f 'ba' -x --audio-format mp3 https://youtu.be/A4lTUgJWNIw
```
Once we have this audio file, we can use amazing tool
[`album-splitter`](https://github.com/crisbal/album-splitter) to split this
big audio file into the audio files; each of them represents a single track.
(`album-splitter` requires a file with time stamps, where you need the initial
mp3 file to be split at. This can be easily found in the description or
comments). Once you have the files, you might want to rename some of them
using already mentioned `rnr` tool. Now, finally, you need to run, for
example, this command in the folder where the files are located:
```bash
ls *.mp3 | fme --ac cover.jpg -y 2023 --at 'Tomorrowland 2023'
```
where `cover.jpg` is the album cover that you would like to put (e.g can be
taken to be the same as thumbnail to the video). 

Viola! Just using a few commands we created an album! We can then send it
whoever we want! :)
