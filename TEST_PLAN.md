# Niri-Bar Comprehensive Test Plan

This document outlines the comprehensive testing strategy for the niri-bar Rust workspace. All tests must be deterministic, hermetic, and provide full coverage of happy paths, error cases, edge cases, and safety checks.

## Testing Philosophy

- **Zero skipped tests**: Every test must run and pass
- **Hermetic testing**: No real filesystem, network, or time dependencies in unit tests
- **Property-based testing**: Use proptest for invariant validation
- **Safety first**: Miri testing for UB detection, Loom for concurrency
- **Coverage ≥80%**: Minimum acceptable line coverage
- **Deterministic**: Fixed seeds for property tests, fake time/clocks

## Module Coverage Matrix

### 🔧 Core Modules

#### 1. `config.rs` - Configuration Management
- **Public APIs**: `ConfigManager`, `NiriBarConfig`, parsing functions
- **Happy Path**: Valid YAML parsing, schema validation, monitor matching
- **Error Cases**: Invalid YAML, schema violations, missing required fields
- **Edge Cases**: Empty configs, malformed patterns, Unicode in paths
- **Properties**: Round-trip parsing, pattern specificity ordering
- **Concurrency**: Config reload thread safety

#### 2. `application.rs` - Main Application Logic
- **Public APIs**: `Application::new()`, `Application::run()`, monitor management
- **Happy Path**: GTK initialization, monitor setup, config loading
- **Error Cases**: GTK init failure, config parse errors, file watcher failures
- **Edge Cases**: No monitors connected, invalid monitor names
- **Properties**: Monitor count consistency, config reload idempotence

#### 3. `monitor.rs` - Monitor Management
- **Public APIs**: `Monitor::new()`, theme updates, column management
- **Happy Path**: Monitor creation, theme switching, column updates
- **Error Cases**: Invalid monitor connectors, GTK failures
- **Edge Cases**: Zero monitors, resolution changes, theme not found

#### 4. `logger.rs` - Logging Infrastructure
- **Public APIs**: `NiriBarLogger::init()`, log configuration
- **Happy Path**: Log file creation, console output, structured logging
- **Error Cases**: File permission errors, invalid log levels
- **Edge Cases**: Log rotation, concurrent writes

#### 5. `ui.rs` - GTK UI Management
- **Public APIs**: UI creation, CSS application, widget management
- **Happy Path**: Widget creation, CSS loading, responsive layouts
- **Error Cases**: CSS parse errors, widget creation failures
- **Edge Cases**: Missing themes, invalid CSS classes

### 📦 Module Registry (`modules/`)

#### 6. `modules/mod.rs` - Module Registry
- **Public APIs**: `create_module_widget()`, module factory functions
- **Happy Path**: Module creation by name, factory lookup
- **Error Cases**: Unknown module names, invalid settings
- **Properties**: Module identifier uniqueness

#### 7. `modules/clock.rs` - Clock Module
- **Public APIs**: `ClockModule::create_widget()`
- **Happy Path**: Time display, custom format strings
- **Error Cases**: Invalid format strings, chrono errors
- **Edge Cases**: Time zone changes, daylight saving
- **Properties**: Time display monotonicity

#### 8. `modules/battery.rs` - Battery Module
- **Public APIs**: `BatteryModule::create_widget()`
- **Happy Path**: Battery status display, power profile management
- **Error Cases**: Missing sysfs files, powerprofilesctl not found
- **Edge Cases**: Battery not present, charging states, percentage boundaries
- **Properties**: Percentage bounds (0-100), icon selection consistency
- **Concurrency**: File monitor race conditions

#### 9. `modules/workspaces.rs` - Workspaces Module
- **Public APIs**: Workspace display functions
- **Happy Path**: Active workspace highlighting, wallpaper switching
- **Error Cases**: Niri IPC connection failures
- **Edge Cases**: Zero workspaces, workspace name changes

#### 10. `modules/window_title.rs` - Window Title Module
- **Public APIs**: Title display with truncation
- **Happy Path**: Title display, length limiting, ellipsization
- **Error Cases**: Unicode truncation issues
- **Edge Cases**: Very long titles, empty titles, special characters

#### 11. `modules/tray.rs` - System Tray Module
- **Public APIs**: Tray icon management
- **Happy Path**: Icon display, click handling
- **Error Cases**: Tray protocol unavailable
- **Edge Cases**: Many tray icons, icon size variations

### 🔌 Integration Modules

#### 12. `file_watcher.rs` - File Monitoring
- **Public APIs**: `FileWatcher::new()`, file change detection
- **Happy Path**: File creation/modification/deletion detection
- **Error Cases**: Permission errors, filesystem issues
- **Edge Cases**: Symlinks, rapid file changes
- **Concurrency**: Multiple watchers, event ordering

#### 13. `wallpaper.rs` - Wallpaper Management
- **Public APIs**: `WallpaperSwitcher::new()`, workspace-based switching
- **Happy Path**: Wallpaper setting via swww, command execution
- **Error Cases**: swww not installed, invalid image paths
- **Edge Cases**: Workspace not found, image loading failures

#### 14. `niri/` - Niri IPC Integration
- **Public APIs**: IPC connection, event streaming
- **Happy Path**: Event parsing, workspace focus detection
- **Error Cases**: Socket connection failures, malformed IPC messages
- **Edge Cases**: IPC protocol version mismatches

## Test Categories by Module

### Unit Tests (White-box)
- Located in `src/**/*.rs` with `#[cfg(test)]` modules
- Test individual functions with mocked dependencies
- Focus on logic correctness, error handling

### Integration Tests (Black-box)
- Located in `tests/` directory
- Test module interactions, file I/O, IPC
- Use hermetic sandboxes (tempfile, assert_fs)

### Property-Based Tests
- Use `proptest` for invariant validation
- Round-trip parsing, boundary condition checking
- Fixed seeds for reproducibility

### Doctests
- Examples in `///` documentation comments
- Must compile and run correctly
- Cover primary use cases

### Concurrency Tests
- Use `loom` for model checking concurrent code
- Test race conditions, deadlocks
- Focus on shared state management

### Safety Tests
- Use `miri` for undefined behavior detection
- Test unsafe code blocks, pointer operations
- Memory safety validation

## Test Infrastructure Requirements

### Dev Dependencies to Add
```toml
[dev-dependencies]
proptest = "1.0"
insta = { version = "1.0", features = ["glob"] }
assert_fs = "1.0"
pretty_assertions = "1.0"
tempfile = "3.0"
tokio = { version = "1.0", features = ["macros", "rt-multi-thread", "time", "test-util"] }
loom = "0.7"
criterion = "0.5"  # For benchmarking
```

### Mock Infrastructure
- **Filesystem**: `assert_fs`, `tempfile` for hermetic file operations
- **Time**: `tokio::time::pause()` for deterministic time testing
- **IPC**: Mock IPC servers for Niri integration testing
- **GTK**: Headless GTK testing or widget mocks

### Test Organization
```
tests/
├── unit/           # Pure unit tests with mocks
├── integration/    # End-to-end with real dependencies
├── property/       # proptest-based invariant tests
├── concurrency/    # loom-based concurrency tests
└── fixtures/       # Test data files
```

## Coverage Goals

### Line Coverage Targets
- **Core modules** (config, application): ≥95%
- **UI modules** (monitor, ui): ≥90%
- **Business logic** (modules/*): ≥85%
- **Integration modules**: ≥80%
- **Overall project**: ≥85%

### Branch Coverage
- **Error paths**: 100% coverage of error handling
- **Edge cases**: All boundary conditions tested
- **Platform-specific code**: All conditional compilation paths

## Safety & Robustness Checks

### Undefined Behavior Detection
- Run `cargo miri test` on all unsafe code
- Test pointer arithmetic, transmutes, raw pointer usage
- Validate memory safety in GTK interactions

### Race Condition Analysis
- Use `loom` to model concurrent scenarios
- Test config reloading under concurrent access
- Validate IPC event handling thread safety

### Memory Leak Detection
- Integration with `valgrind` or `asan`
- Test long-running scenarios
- GTK widget lifecycle management

### Panic Safety
- Test panic unwinding in critical paths
- Validate cleanup on panic (file handles, threads)
- Poisoned mutex handling

## Performance Testing

### Benchmarks
- Configuration parsing performance
- Module creation overhead
- IPC message processing speed

### Load Testing
- Many monitors/monitors with many modules
- High-frequency IPC events
- Large configuration files

## CI/CD Integration

### Automated Test Pipeline
1. **Linting**: `cargo clippy -- -D warnings`
2. **Formatting**: `cargo fmt --check`
3. **Unit Tests**: `cargo test --lib`
4. **Integration Tests**: `cargo test --test '*'`
5. **Doctests**: `cargo test --doc`
6. **Property Tests**: `cargo test --features proptest`
7. **Concurrency Tests**: `cargo test --features loom`
8. **Coverage**: `cargo llvm-cov --lcov`
9. **Safety**: `cargo miri test`
10. **Mutation**: `cargo mutants`

### Quality Gates
- **Zero warnings**: Clippy must pass clean
- **No unsafe without review**: All unsafe blocks documented
- **Coverage threshold**: Fail CI if below 80%
- **Miri clean**: No UB findings
- **Deterministic tests**: No flaky test runs

## Test Maintenance

### Regression Prevention
- Snapshot tests for complex outputs
- Property tests for invariant preservation
- Integration tests for end-to-end workflows

### Documentation
- Test README with running instructions
- Coverage reports in CI artifacts
- Performance regression tracking

This plan ensures comprehensive, maintainable test coverage that catches bugs early and prevents regressions while maintaining high code quality standards.
