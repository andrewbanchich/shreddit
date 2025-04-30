# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.1](https://github.com/andrewbanchich/shreddit/compare/v1.1.0...v1.1.1) - 2025-04-30

### Fixed

- use consistent sorting to ensure all comments captured ([#180](https://github.com/andrewbanchich/shreddit/pull/180))

### Other

- Update usage output to document valid arguments ([#179](https://github.com/andrewbanchich/shreddit/pull/179))
- bump chrono from 0.4.40 to 0.4.41 ([#182](https://github.com/andrewbanchich/shreddit/pull/182))
- bump parse_datetime from 0.8.0 to 0.9.0 ([#181](https://github.com/andrewbanchich/shreddit/pull/181))
- bump tokio from 1.44.1 to 1.44.2 ([#173](https://github.com/andrewbanchich/shreddit/pull/173))
- bump clap from 4.5.36 to 4.5.37 ([#177](https://github.com/andrewbanchich/shreddit/pull/177))
- bump clap from 4.5.33 to 4.5.36 ([#174](https://github.com/andrewbanchich/shreddit/pull/174))
- bump clap from 4.5.32 to 4.5.33 ([#170](https://github.com/andrewbanchich/shreddit/pull/170))
- bump reqwest from 0.12.14 to 0.12.15 ([#168](https://github.com/andrewbanchich/shreddit/pull/168))
- bump async-trait from 0.1.87 to 0.1.88 ([#167](https://github.com/andrewbanchich/shreddit/pull/167))
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
