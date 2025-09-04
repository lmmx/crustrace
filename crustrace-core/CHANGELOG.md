# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.9](https://github.com/lmmx/crustrace/compare/crustrace-core-v0.1.8...crustrace-core-v0.1.9) - 2025-09-04

### <!-- 1 -->Features

- *(debug)* crate feature for easy stringification; fix HTML escaping ([#21](https://github.com/lmmx/crustrace/pull/21))

## [0.1.8](https://github.com/lmmx/crustrace/compare/crustrace-core-v0.1.7...crustrace-core-v0.1.8) - 2025-09-04

### <!-- 1 -->Features

- Mermaid diagram generation; integration tests ([#18](https://github.com/lmmx/crustrace/pull/18))

## [0.1.7](https://github.com/lmmx/crustrace/compare/crustrace-core-v0.1.6...crustrace-core-v0.1.7) - 2025-06-11

### <!-- 4 -->Documentation

- module docstring (again)

## [0.1.6](https://github.com/lmmx/crustrace/compare/crustrace-core-v0.1.5...crustrace-core-v0.1.6) - 2025-06-11

### <!-- 4 -->Documentation

- module docstring

## [0.1.5](https://github.com/lmmx/crustrace/compare/crustrace-core-v0.1.4...crustrace-core-v0.1.5) - 2025-06-10

### <!-- 1 -->Features

- set span `parent` via instrument `parent` ([#16](https://github.com/lmmx/crustrace/pull/16))
- set span `target` via instrument `target` ([#15](https://github.com/lmmx/crustrace/pull/15))

### <!-- 5 -->Refactor

- split out unit tests as submodules

## [0.1.4](https://github.com/lmmx/crustrace/compare/crustrace-core-v0.1.3...crustrace-core-v0.1.4) - 2025-06-09

### <!-- 1 -->Features

- *(ret)* handle `Debug` and `Display` args to `ret` ([#12](https://github.com/lmmx/crustrace/pull/12))

### <!-- 6 -->Testing

- unignore
- tidy tests ([#13](https://github.com/lmmx/crustrace/pull/13))

## [0.1.3](https://github.com/lmmx/crustrace/compare/crustrace-core-v0.1.2...crustrace-core-v0.1.3) - 2025-06-08

### <!-- 1 -->Features

- trace return when ret is passed ([#11](https://github.com/lmmx/crustrace/pull/11))
- trace values, more complete function parsing ([#10](https://github.com/lmmx/crustrace/pull/10))

## [0.1.2](https://github.com/lmmx/crustrace/compare/crustrace-core-v0.1.1...crustrace-core-v0.1.2) - 2025-06-07

### <!-- 5 -->Refactor

- use declarative unsynn parsing not imperative style ([#9](https://github.com/lmmx/crustrace/pull/9))

## [0.1.1](https://github.com/lmmx/crustrace/compare/crustrace-core-v0.1.0...crustrace-core-v0.1.1) - 2025-06-07

### <!-- 2 -->Bug Fixes

- amend README metadata

### <!-- 7 -->Build System and CI

- README meta

### <!-- 9 -->Other

- license
- publishable crate yet?
- another one
- please accept my humble README o crates.io
- amend

## [0.1.0](https://github.com/lmmx/crustrace/releases/tag/crustrace-core-v0.1.0) - 2025-05-27

### <!-- 1 -->Features

- support trait default methods; tests for all cases in README ([#4](https://github.com/lmmx/crustrace/pull/4))
- support impl Struct methods
- *(absolve)* free of `syn` at last ([#2](https://github.com/lmmx/crustrace/pull/2))
- *(attributes)* replace `tracing-attributes` crate (use unsynn) ([#1](https://github.com/lmmx/crustrace/pull/1))
- working for mod level blocks too
- Initial commit

### <!-- 5 -->Refactor

- simplify code, deduplicate where possible ([#5](https://github.com/lmmx/crustrace/pull/5))
- rename 'trace all' to 'omni', it's cleaner
- *(core)* move trace all impl into omnibus module
- merge crustrace-attributes crate into crustrace-core ([#3](https://github.com/lmmx/crustrace/pull/3))

### <!-- 7 -->Build System and CI

- *(core)* publish the core too

### <!-- 8 -->Styling

- prefer core to std
- lint

### <!-- 9 -->Other

- set up git hooks
