# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.1](https://github.com/andrewbanchich/shreddit/compare/v1.1.0...v1.1.1) - 2025-03-15

### Other

- bump reqwest from 0.12.13 to 0.12.14 ([#165](https://github.com/andrewbanchich/shreddit/pull/165))
- bump tokio from 1.44.0 to 1.44.1 ([#166](https://github.com/andrewbanchich/shreddit/pull/166))
- bump reqwest from 0.12.12 to 0.12.13 ([#164](https://github.com/andrewbanchich/shreddit/pull/164))
- bump clap from 4.5.31 to 4.5.32
- bump serde from 1.0.218 to 1.0.219 ([#160](https://github.com/andrewbanchich/shreddit/pull/160))

## [1.1.0](https://github.com/andrewbanchich/shreddit/compare/v1.0.2...v1.1.0) - 2025-03-09

### Added

- support relative timestamps in --after filters

### Other

- update doc comment for relative timestamps
- rename variable

## [1.0.2](https://github.com/andrewbanchich/shreddit/compare/v1.0.1...v1.0.2) - 2025-03-09

### Fixed

- make --before filter optional
- make --after filter optional
- reddit link

### Other

- Revert "ci: allow workflow_dispatch trigger for release"

## [1.0.1](https://github.com/andrewbanchich/shreddit/compare/v1.0.0...v1.0.1) - 2025-03-09

### Other

- allow workflow_dispatch trigger for release
- use bash for windows release
- update checks workflow
