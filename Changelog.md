# Change Log

All notable changes to this project will be documented in this file. This file adheres to the format of [keep a changelog.](https://keepachangelog.com/en/1.0.0/)

## [Unreleased]

### Changed:

* Updating Cargo dependencies
* Updated binary name to 'eagle-plm' to match the repo
* Moved main to bin folder

### Removed:
* Removed flags for filename entry and part_number entry (as they're default and required every time)

### Added

* Changelog!
* Added export BOM function
* Added export shortages function
* Added export shortages command
* Added update_from_file in inventory
* Added inventory update command
* Added option to export_all for export_to_file. Now, by default, only exports quantity > 0
* Inventory show only shows parts with quantity > 0