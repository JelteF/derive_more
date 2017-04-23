# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).


## [Unreleased]

### Changed
- Deriving `From`, `Into` and `Constructor` now works for empty structs.


## 0.6.1 - 2017-03-08

### Changed
- The `new()` method that is created when deriving `Constructor` is now public.
  This makes it a lot more useful.


## 0.6.0 - 2017-02-20

### Added

- Derives for `Into`, `Constructor` and `MulAssign`-like

### Changed

- `From` is now derived for enum variants with multiple fields.

### Fixed

- Derivations now support generics.


## 0.5.0 - 2017-02-02

### Added

- Lots of docs.
- Derives for `Neg`-like and `AddAssign`-like.

### Changed
- `From` can now be derived for structs with multiple fields.
