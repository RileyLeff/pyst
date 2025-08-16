# Pyst Testing Infrastructure

This directory contains comprehensive testing infrastructure for pyst using a **hybrid approach** combining native testing with containerized integration testing.

## Testing Strategy

### Hybrid Approach
We use two complementary testing strategies:

1. **Native Tests** (All Platforms)
   - Test compilation and basic runtime behavior across platforms
   - Catch platform-specific issues (file paths, linking, native dependencies)
   - Fast feedback for basic functionality
   - Run on: macOS Intel, macOS ARM64, Windows x64, Linux x64, Linux ARM64

2. **Container Tests** (Linux Only)  
   - Comprehensive testing of pyst's core logic in isolated environments
   - Test complex workflows with real Python scripts and dependencies
   - Consistent, reproducible results regardless of host environment
   - Run only on Linux since containers normalize the environment

### Why This Approach?
- **Efficiency**: Avoids running identical containerized tests on 5 different platforms
- **Coverage**: Native tests catch platform-specific compilation/runtime issues  
- **Reliability**: Container tests ensure pyst's core logic works consistently
- **Speed**: Optimized Docker images + reduced redundancy = faster CI

### Platform-Specific Notes
- **Linux ARM64**: Uses vendored OpenSSL (`OPENSSL_VENDORED=1`) for cross-compilation to avoid linking issues
- **Windows**: Native tests catch Windows-specific path handling and file system behavior
- **macOS**: Tests both Intel and Apple Silicon architectures natively
- **Code Coverage**: Focuses on library code only, excludes integration tests for simplicity

## Test Organization

### Unit Tests
- Located in individual modules alongside source code
- Fast execution, no external dependencies
- Run with: `cargo test --lib`

### Integration Tests
- **Location**: `tests/integration/`
- **Container Tests**: `tests/containers/`
- **Test Fixtures**: `tests/fixtures/`

### Test Types

#### Container Infrastructure Tests (`containers/`)
- **Basic Setup**: Container creation, volume mounting, command execution
- **Tool Installation**: Rust, uv, system dependencies
- **Binary Building**: End-to-end pyst compilation

#### Integration Tests (`integration/`)
- **Script Execution**: uv-based Python script running
- **Argument Forwarding**: Click framework, complex argument handling  
- **Working Directory**: CWD behavior, path resolution
- **End-to-End**: Complete pyst workflow testing

## Docker Infrastructure

### Optimized Test Image
- **Dockerfile**: `containers/Dockerfile`
- **Image Tag**: `pyst-test:latest`
- **Pre-installed**: Python 3.11, Rust, uv, build tools, OpenSSL
- **Build**: `cd containers && docker build -t pyst-test:latest .`

### Performance Benefits
- **Manual Tool Installation**: ~45 seconds setup time
- **Optimized Image**: ~0 seconds setup time
- **Total Test Speedup**: 60%+ faster execution

## Running Tests

### Quick Tests (No Containers)
```bash
cargo test --lib --bins
```

### Container Tests
```bash
# Build optimized image first
cd pyst/tests/containers
docker build -t pyst-test:latest .

# Run container infrastructure tests
cargo test --package pyst containers

# Run integration tests  
cargo test --package pyst integration
```

### Specific Test Categories
```bash
# Volume mounting and basic functionality
cargo test --package pyst test_source_mounting
cargo test --package pyst test_fixtures_mounting

# Script execution with uv
cargo test --package pyst test_pyst_scripts_with_uv
cargo test --package pyst test_argument_forwarding

# Working directory behavior
cargo test --package pyst test_working_directory_behavior

# Full end-to-end (slower)
cargo test --package pyst test_full_dev_setup

# Optimized workflow (faster)  
cargo test --package pyst test_optimized_pyst_workflow
```

## Test Fixtures

### Simple Project (`fixtures/simple-project/`)
- **hello.py**: Basic PEP 723 script with dependencies
- **test-cwd.py**: Working directory testing script
- **Configuration**: `.pyst.toml` for pyst settings

### Click Project (`fixtures/click-project/`)
- **cli-script.py**: Complex CLI with Click framework
- **Arguments**: Flags, options, positional args, extra args
- **Testing**: Argument forwarding, framework integration

## CI/CD Integration

### GitHub Actions Workflows

#### Main CI (`ci.yml`)
- **Platforms**: macOS Intel, macOS ARM64, Windows x64, Linux x64, Linux ARM64
- **Quick Checks**: Format, clippy, basic compilation
- **Native Tests**: Platform-specific builds and unit tests
- **Integration Tests**: Container-based comprehensive testing
- **End-to-End**: Full workflow validation (on push/label)

#### Docker Build (`docker.yml`)
- **Multi-Platform**: AMD64, ARM64 Docker image builds
- **Image Testing**: Tool verification across architectures
- **Caching**: GitHub Actions cache for faster builds

### Platform Support Matrix

| Platform | Native Build | Unit Tests | Integration Tests | Notes |
|----------|-------------|------------|------------------|-------|
| macOS Intel | ✅ | ✅ | ⚠️ | Docker required for integration |
| macOS ARM64 | ✅ | ✅ | ⚠️ | Docker required for integration |
| Windows x64 | ✅ | ✅ | ⚠️ | Docker Desktop required |
| Linux x64 | ✅ | ✅ | ✅ | Full support |
| Linux ARM64 | ✅ | ⚠️ | ✅ | Cross-compilation |

## Container Helpers

### PystContainer
- **Base Images**: `python_base()`, `pyst_dev_base()`
- **With Mounts**: `pyst_with_source()`, `pyst_with_fixtures()`
- **Optimized**: `optimized_full_dev()`, `optimized_with_fixtures()`

### PystSetup  
- **Tool Installation**: `install_rust()`, `install_uv()`
- **Binary Building**: `build_pyst()`, `full_setup()`

### CommandResult
- **Execution**: `exec()` with stdout/stderr/exit_code capture
- **Assertions**: Success checking, output validation

## Development Workflow

### Adding New Tests
1. Create test fixtures in appropriate `fixtures/` subdirectory
2. Use existing container helpers or extend `PystContainer`
3. Follow existing test patterns for consistency
4. Run locally before CI

### Debugging Test Failures
```bash
# Verbose output
cargo test test_name -- --nocapture

# Single test with timing
cargo test test_name --

# Docker debugging
docker run -it --rm pyst-test:latest bash
```

### Performance Optimization
- Use `optimized_*` container methods for faster tests
- Prefer unit tests over integration tests when possible
- Cache Docker layers effectively
- Consider test parallelization

## Troubleshooting

### Common Issues
1. **Docker not running**: Start Docker Desktop/daemon
2. **Image not found**: Build `pyst-test:latest` image first
3. **Permission errors**: Check Docker permissions
4. **Slow tests**: Use optimized containers, check Docker resources
5. **Cross-platform**: Verify Docker multi-platform support

### Performance Tips
- Use optimized Docker image for 60%+ speedup
- Run integration tests in parallel when possible
- Cache Rust dependencies between test runs
- Clean up Docker resources periodically

This testing infrastructure provides comprehensive, reliable, and fast validation of pyst functionality across multiple platforms and scenarios.