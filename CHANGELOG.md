# Changelog

All _notable_ changes to this project will be documented in this file.

The format is based on _[Keep a Changelog][keepachangelog]_,
and this project adheres to a _modified_ form of _[Semantic Versioning][semver]_
(major version is the year; minor and patch are the same).

## [Unreleased]

### Added

- Make CLI more generic, any Rust project that depends on stelae can now re-export CLI ([#89])
- Add `from_path` associated function to init `Repo` ([#89])
- Add `_archive` endpoint ([#81])
- Restrict `_archive` endpoint access on private repositories ([#81])

### Changed

- Bump rust-version to `1.88` ([#86])

### Fixed

### Removed

[#89]: https://github.com/openlawlibrary/stelae/pull/89
[#86]: https://github.com/openlawlibrary/stelae/pull/86
[#81]: https://github.com/openlawlibrary/stelae/pull/81

## [0.5.1]

### Added

- Add `--include/-i` and `--exclude/-e` to `stelae update` ([#88])

### Changed

### Fixed

- Fix(history, changes): skip iterating over publication when revoked ([#88])

### Removed

[#88]: https://github.com/openlawlibrary/stelae/pull/88

## [0.5.0]

### Added

- Add `build_date` and `codified_date` columns to `data_repo_commits` table ([#76])
- Add `X-File-Path` header to `_stelae` and git microserver HTTP responses ([#70])
- Added tests fot `stelae git` ([64])

### Changed

- Merged `stelae git` and `stelae serve` into single command ([#64])

### Fixed

- git serve now support committish that contains / in name ([#64])

### Removed

[#76]: https://github.com/openlawlibrary/stelae/pull/76
[#70]: https://github.com/openlawlibrary/stelae/pull/70
[#64]: https://github.com/openlawlibrary/stelae/pull/64

## [0.4.2]

### Added

### Changed

### Fixed

- Turn error into a warning when stele rdf repo is not setup ([#75])
- Fix insert hashes on non-revoked publications ([#73])

### Removed

[#75]: https://github.com/openlawlibrary/stelae/pull/75
[#73]: https://github.com/openlawlibrary/stelae/pull/73


## [0.4.1]

### Added

### Changed

### Fixed

- Add missing route to versions endpoint ([#68])

### Removed

[#68]: https://github.com/openlawlibrary/stelae/pull/68

## [0.4.0]

### Added

- Insert commit hashes in the database ([63])

### Changed

- Allow HEAD requests for dynamic routes ([58])
- Rename `.stelae` to `.taf` dir ([61])
- Bump rust-version to `1.83` ([61])

### Fixed

### Removed

[#63]: https://github.com/openlawlibrary/stelae/pull/63
[#61]: https://github.com/openlawlibrary/stelae/pull/61
[#58]: https://github.com/openlawlibrary/stelae/pull/58

## [0.3.2]

### Added

### Changed

### Fixed

- Fix `stelae update` partial update when there are new publications ([#56])

### Removed

[#56]: https://github.com/openlawlibrary/stelae/pull/56

## [0.3.1]

### Added

### Changed

### Fixed

- Fix resolve to `_api/versions` requests without trailing `/` ([#52])

### Removed

[#52]: https://github.com/openlawlibrary/stelae/pull/52

## [0.3.0]

### Added

- Add filesystem logging ([#42])
- Add command to insert history into database from RDF ([#33], [#42], [#44], [#46])
- Add versions endpoint to view dates on which documents and/or collections have changed ([#33])
- Add command to serve current documents from repositories ([#32])

### Changed

### Fixed

- Load paths to git repositories at start-time ([#47])
- Fixes to insert history command ([#46])

### Removed

[#47]: https://github.com/openlawlibrary/stelae/pull/47
[#46]: https://github.com/openlawlibrary/stelae/pull/46
[#44]: https://github.com/openlawlibrary/stelae/pull/44
[#42]: https://github.com/openlawlibrary/stelae/pull/42
[#33]: https://github.com/openlawlibrary/stelae/pull/33
[#32]: https://github.com/openlawlibrary/stelae/pull/32

## [0.2.1]

### Added

- Add basic instruments to git serve ([#26])

### Changed

### Fixed

- Fix safe directory issues by upgrading git2 to latest version ([#26])

### Removed

[#26]: https://github.com/openlawlibrary/stelae/pull/26

## [0.2.0]

### Added

### Changed

- Update license, release under AGPL ([#25])

### Fixed

### Removed

[#25]: https://github.com/openlawlibrary/stelae/pull/25

## [0.1.1]

### Added

### Changed

### Fixed

- default to `text/html` mime-type instead of `application/octet-stream`

### Removed

## [0.1.0]

### Added

- Added `/{namespace}/{name}/{commitish}{remainder}` endpoint (initial commits)
- Added ci/cd and local verbose clippy in vscode

### Changed

### Fixed

### Removed

[Unreleased]: https://github.com/openlawlibrary/stelae/compare/v0.5.1...HEAD
[0.5.1]: https://github.com/openlawlibrary/stelae/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/openlawlibrary/stelae/compare/v0.4.2...v0.5.0
[0.4.2]: https://github.com/openlawlibrary/stelae/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/openlawlibrary/stelae/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/openlawlibrary/stelae/compare/v0.3.2...v0.4.0
[0.3.2]: https://github.com/openlawlibrary/stelae/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/openlawlibrary/stelae/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/openlawlibrary/stelae/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/openlawlibrary/stelae/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/openlawlibrary/stelae/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/openlawlibrary/stelae/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/openlawlibrary/stelae/compare/2b01423c06369f5f0f168ae4c4698371d713ede7...v0.1.0

[keepachangelog]: https://keepachangelog.com/en/1.0.0/
[semver]: https://semver.org/spec/v2.0.0.html