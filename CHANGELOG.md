# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2025-06-20

### Added
- Enhanced API consistency and usability improvements

### Changed
- **BREAKING**: Removed builtin node types to simplify the codebase
- Simplified action enum structure for better performance and maintainability
- Improved overall API design and consistency

### Fixed
- Updated documentation to reflect recent changes and improvements

## [0.3.0] - 2025-06-16

### Added
- New unified SharedStore example demonstrating custom storage backends
- Enhanced documentation with comprehensive getting-started guide
- Improved example projects with better code organization

### Changed
- **BREAKING**: Unified StorageBackend trait and SharedStore structure
- **BREAKING**: Replaced hidden terminal actions with explicit terminal routes
- **BREAKING**: Migrated from `storage` module to `shared_store` module
- Improved flow macro syntax and functionality
- Enhanced type safety and error handling throughout the codebase
- Better separation of concerns between storage backends and shared store

### Fixed
- Made async-openai, futures, and rand dependencies optional for better compile-time configuration
- Improved feature gate system to reduce compile dependencies when not needed

## [0.2.1] - 2025-06-16

### Fixed
- Made async-openai, futures, and rand dependencies optional for better compile-time configuration
- Improved feature gate system to reduce compile dependencies when not needed

## [0.2.0] - 2025-06-13

### Added
- Flow macro to simplify generate flows
- Some simple examples
- Redis storage backend support
- Terminal action warning system in FlowBuilder

### Changed
- **BREAKING**: Unified node and NodeBackend
- Better documentation structure and organization

## [0.1.0] - 2025-06-12

### Added
- Initial release of CosmoFlow workflow engine
- Core flow orchestration system
- Basic node execution framework
- Memory and file storage backends
- Built-in node types (Log, SetValue, GetValue, Delay)
- Shared store for data communication
- Action system for flow control
- Basic example projects
- Initial documentation

---

For more details about any release, please check the [GitHub releases page](https://github.com/echozyr2001/CosmoFlow/releases).