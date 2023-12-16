# Changelog

## 0.2.0

- Fixed parsing problems when last event of SubRip file contained a blank line
- Fixed WebVTT parsing logic to skip over invalid/unparsable blocks instead of stopping prematurely
- Made adjustment of timings triggered by framerate update more accurate for timed MicroDVD subtitles
- Remove `Sized` as supertrait of `Subtitle`
- Add `as_plaintext()` method to `TextEvent` trait
- Rename Error variant `UnknownFileTypeError` to `FormatUnknownError`

## 0.1.0

- Initial release
