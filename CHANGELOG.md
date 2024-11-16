# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.1] & [0.5.2] & [0.5.3] (based on `1.4.0b.158d65c`)
### Fixed
- Fixed small bug in building docs for `docs.rs`.

## [0.5.0] (based on `1.4.0b.158d65c`)
### Added
- libloading support for loading the `ultralight` shared library at runtime (#5 & #8).

## [0.4.1] (based on `1.4.0b.158d65c`)
### Fixed
- Bug in `glium` built in driver where it will crash when creating some textures (#6)

## [0.4.0] (based on `1.4.0b.158d65c`)
### Added
- `ImageSource` API for loading textures and custom bitmaps as html images.
- More types of `view::ConsoleMessageSource` supported by the latest `ultralight` version.

## [0.3.0] (based on `1.4.0-beta.ae79344`) - 2024-09-07
### Added
- `View::create_local_inspector_view` to create a local inspector view.
- `display_id` functionalities to `View`.
- `Gamepad` events.
- `Renderer::start_remote_inspector_server` for starting a remote inspector server.
- Custom `FontLoader` support.


## [0.2.0] - 2024-02-06
### Changed
- Updated `glium` to `0.34`
    - Breaking, since `glium` types are incompatible with older versions.

## [0.1.4] - 2024-02-06

### Added
- The initial implementation (most) of the library. This is the first version we added `CHANGELOG`, so starting from here :).
- Implementation of most of the functionalities of the original C/C++ API.
- Added examples for the library.
- Added implementation for `glium` driver using OpenGL.

[unreleased]: https://github.com/Amjad50/ul-next/compare/v0.5.2...HEAD
[0.5.3]: https://github.com/Amjad50/ul-next/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/Amjad50/ul-next/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/Amjad50/ul-next/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/Amjad50/ul-next/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/Amjad50/ul-next/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/Amjad50/ul-next/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/Amjad50/ul-next/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/Amjad50/ul-next/compare/v0.1.4...v0.2.0
[0.1.4]: https://github.com/Amjad50/ul-next/compare/f937902...v0.1.4
