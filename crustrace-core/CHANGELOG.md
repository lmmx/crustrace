# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
