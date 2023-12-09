# aspasia

**aspasia is a subtitle parsing library written in Rust that offers various functionality for working with subtitles.**

Parsing of formats is done somewhat loosely, meaning that it will make a best effort to parse subtitle files as the given format, even if they don't fit the specification perfectly. Parsing incorrectly formatted subtitle files will result in an empty object instead of returning any errors.

## Features

- Parse subtitle files from a variety of different formats:
  - SubRip (.srt)
  - Advanced SubStation Alpha V4+ (.ass)
  - SubStation Alpha V4 (.ssa)
  - WebVTT (.vtt)
  - MicroDVD (.sub)
- Detect subtitle format automatically based on file extension and file contents
- Convert between subtitle formats while preserving compatible formatting and information
- Strip formatting information from subtitles
- Use utility methods to shift subtitle times, calculate the duration of individual events, and more

## Documentation

https://docs.rs/aspasia/latest/aspasia/

## Usage

### Installation

You can add aspasia to your project using Cargo: `cargo add aspasia`

...or you can edit your `Cargo.toml` to add:

```toml
[dependencies]
aspasia = "0.1"
```

### Examples

**Basic usage**

```rust
use aspasia::{SubRipSubtitle, Subtitle, TimedEvent, TimedSubtitleFile, WebVttSubtitle};

// We can directly specify the format to open a subtitle file
let vtt = WebVttSubtitle::from_path("/path/to/some.vtt")?;

// and then directly work with its data
println!("{}", vtt.header().cloned().unwrap_or_default());

// or we could use the more general interface to open (timed) subtitle files
let sub = TimedSubtitleFile::new("/path/to/file.srt")?;

// Move the underlying data out in order to access format-specific properties
// Note that if the format doesn't match, this will perform a conversion instead of just moving the data
let mut srt = SubRipSubtitle::from(sub);

// Now we can access format-specific methods like SubRipSubtitle::renumber()
srt.renumber();

// Access and modify events
for event in srt.events_mut() {
    event.shift(600.into());
}

// Write the modified subtitle to file
srt.export("/path/to/output.srt")?;
```

**Format conversion**

```rust
use aspasia::{AssSubtitle, SubRipSubtitle, Subtitle, TimedSubtitleFile, WebVttSubtitle};

let sub = TimedSubtitleFile::new("/path/to/file.srt")?;

// Get the file as its specific format
let srt = SubRipSubtitle::from(sub);

// You can use into() to convert the file
let vtt: WebVttSubtitle = srt.into();

// or from()
let ass = AssSubtitle::from(vtt);

ass.export("/path/to/converted.ass")?;
```

See [examples](examples) folder for more

## Roadmap

aspasia is currently under heavy development, and its API is not particularly stable. Once both API and functionality are relatively stable, I would like to release a 1.0 version. To that end, any feedback, bug reports, or other forms of contribution are very welcome.

### Planned

- Improved conversion
- Support for more subtitle formats
- More utility methods for working with subtitles/subtitle events
- Better test coverage
- Better documentation
- serde feature?

## License

aspasia is [licensed](LICENSE) under the [BSD 0-Clause](https://opensource.org/license/0bsd/) license.
