# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project doesn't adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

If you're looking for the changes included in the latest beta (against the latest stable version), check the unreleased section.

## [0.9.9]
### Fixed
- Fixed error when saving patched pack.
- Fixed error decoding compressed files with a very specific file size.

## [0.9.8]
### Changed
- Updated RPFM lib to 4.4.2, to enable support for compressed packs.
- Databases now cache the vanilla data, so speed up launching the game a lot.

## [0.9.7]
### Changed
- SQL script processing should be a lot faster now.

## [0.9.6]
### Added
- Implemented support for INSERT INTO SQL statements.

### Changed
- DB is now in the config folder.

## [0.9.5]
### Fixed
- Fixed autoupdater version check.

## [0.9.4]
### Added
- Implemented "Enable Dev-Only UI" feature.
- Implemented support for parametrized SQL scripts.

### Changed
- The sqlite database now is cleared properly.

### Fixed
- Fixed an issue that caused changes made by SQL scripts to not show up ingame.
- Fixed an issue that caused movie packs in data to not be taken into account as part of the load order.

## [0.9.3]
### Added
- Implemented experimental support for SQL scripts.

## [0.9.2]
### Added
- Implemented "Remove Siege Attacker" feature. Only supported in Warhammer 3.

## [0.9.1]
### Changed
- Nothing. This release is for testing the self-updater.

## [0.9.0]
### Changed
- Initial Release

[Unreleased]: https://github.com/Frodo45127/twpatcher/compare/v0.9.9...HEAD
[0.9.9]: https://github.com/Frodo45127/twpatcher/compare/v0.9.8...v0.9.9
[0.9.8]: https://github.com/Frodo45127/twpatcher/compare/v0.9.7...v0.9.8
[0.9.7]: https://github.com/Frodo45127/twpatcher/compare/v0.9.6...v0.9.7
[0.9.6]: https://github.com/Frodo45127/twpatcher/compare/v0.9.5...v0.9.6
[0.9.5]: https://github.com/Frodo45127/twpatcher/compare/v0.9.4...v0.9.5
[0.9.4]: https://github.com/Frodo45127/twpatcher/compare/v0.9.3...v0.9.4
[0.9.3]: https://github.com/Frodo45127/twpatcher/compare/v0.9.2...v0.9.3
[0.9.2]: https://github.com/Frodo45127/twpatcher/compare/v0.9.1...v0.9.2
[0.9.1]: https://github.com/Frodo45127/twpatcher/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/Frodo45127/twpatcher/tree/v0.9.0

