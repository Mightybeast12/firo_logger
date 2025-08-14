# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Complete GitHub Actions CI/CD pipeline
- Automated dependency updates with Dependabot
- Auto-merge for patch and minor dependency updates
- Code coverage reporting
- Security audit checks

### Changed
- Improved dual singleton/instance pattern implementation
- Enhanced error handling and documentation

### Fixed
- All compiler warnings resolved
- Doc test examples corrected
- Thread-safe logger implementation

## [0.3.0] - 2025-08-14

### Added
- Dual singleton/instance pattern for better testing
- Thread-local scoped logging support
- Independent logger instances
- Enhanced test isolation
- Comprehensive integration tests

### Changed
- Refactored logger architecture for better concurrency
- Improved configuration validation
- Updated documentation with new usage patterns

### Fixed
- Concurrent testing issues resolved
- Memory safety improvements
- Performance optimizations

## [0.2.0] - Previous Release

### Added
- Async logging support
- File rotation capabilities
- JSON structured logging
- Custom metadata support
- Module-specific filtering

### Changed
- Enhanced configuration system
- Improved error handling

## [0.1.0] - Initial Release

### Added
- Basic logging functionality
- Colored console output
- File logging
- Level-based filtering
- Thread-safe operations

---

## Release Process

To create a new release:

1. Update the version in `Cargo.toml`
2. Update this CHANGELOG.md with the new version
3. Commit changes: `git commit -m "chore: release v<version>"`
4. Create and push a tag: `git tag v<version> && git push origin v<version>`
5. GitHub Actions will automatically publish to crates.io and create a GitHub release

## Dependency Updates

- Dependencies are automatically updated weekly by Dependabot
- Patch and minor updates are auto-merged if CI passes
- Major updates require manual review
- Version bumps for dependency updates are handled automatically
