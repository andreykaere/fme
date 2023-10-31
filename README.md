This is fme — flexible metadata editor.


## Usage

### Options

... <help message>

<!-- Think about two different modes: regex and special parsing -->


It is recommended to use [`rnr`](https://github.com/ismaelgv/rnr) utility to
rename files beforehand if you need it (it might be useful if you download it,
for example, from some youtube playlist and you want it to format first,
before using `--parse` option).


## Examples

### Basic examples

- Default mode

- Specified mode

- Unknown to the parser pattern, hence have to be specified manually


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
ls *.mp3 | fme -p '{n} {a} - {t}' --ac cover.jpg -y 2023 --at 'Tomorrowland 2023'
```
where `cover.jpg` is the album cover that you would like to put (e.g can be
taken to be the same as thumbnail to the video). 

Viola! Just using a few commands we created an album! We can then send it
whoever we want! :)
