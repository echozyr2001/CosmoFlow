# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.1] - 2025-06-26

### Added
- New `async_flow!` macro for creating async workflows with cleaner syntax
- Comprehensive test coverage for `async_flow!` macro functionality

### Fixed
- Fixed compiler warnings when using `flow!` macro in sync-only configurations
- Resolved conditional compilation issues with flow macros in different feature combinations

## [0.5.0] - 2025-06-24

### Added
- Loop flow support for iterative workflow patterns with self-routing capabilities
- New cookbook directory with production-ready examples and real-world solutions
- Enhanced feature configuration system with better granularity and modularity
- Async workflow support as optional feature for better compile-time optimization
- New convenience feature combinations: `minimal`, `basic`, `standard`, and `full`

### Changed
- **BREAKING**: Restructured project organization - moved complex examples to `cookbook/`
- **BREAKING**: Made async support optional with dedicated `async` feature flag
- **BREAKING**: Changed default features to `basic` (storage-memory only) for better UX
- Enhanced storage backend feature organization with individual feature flags
- Improved example structure with separate simple examples and production cookbook
- Better separation between sync and async APIs for reduced compilation overhead

### Fixed
- Improved compile-time dependency management with optional features
- Better feature gate organization for optional dependencies
- Reduced binary size for applications not using all features

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