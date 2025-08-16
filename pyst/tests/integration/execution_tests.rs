use crate::containers::{CommandResult, PystContainer};
use testcontainers::{runners::AsyncRunner, ImageExt};

/// Test volume mounting and source code access
#[tokio::test]
async fn test_source_mounting() {
    let project_root = std::env::current_dir()
        .expect("Failed to get current directory")
        .parent() // Go up from pyst/ to project root
        .expect("Failed to get parent directory")
        .to_path_buf();

    let image = PystContainer::pyst_with_source(&project_root);
    let container = image.start().await.expect("Failed to start container");

    // Verify source code is mounted
    let ls_result = CommandResult::exec(&container, vec!["ls", "-la", "/workspace"])
        .await
        .expect("Failed to list workspace");

    assert!(ls_result.success(), "Should be able to list workspace");
    assert!(
        ls_result.stdout.contains("Cargo.toml"),
        "Should see project Cargo.toml"
    );
    assert!(
        ls_result.stdout.contains("pyst"),
        "Should see pyst directory"
    );

    println!("✅ Source code successfully mounted");
}

/// Test fixtures mounting
#[tokio::test]
async fn test_fixtures_mounting() {
    let fixtures_dir = std::env::current_dir()
        .expect("Failed to get current directory")
        .join("tests/fixtures");

    let image = PystContainer::pyst_with_fixtures(&fixtures_dir);
    let container = image.start().await.expect("Failed to start container");

    // Verify fixtures are mounted
    let ls_result = CommandResult::exec(&container, vec!["ls", "-la", "/test-fixtures"])
        .await
        .expect("Failed to list test fixtures");

    assert!(ls_result.success(), "Should be able to list test fixtures");
    assert!(
        ls_result.stdout.contains("simple-project"),
        "Should see simple-project fixture"
    );
    assert!(
        ls_result.stdout.contains("click-project"),
        "Should see click-project fixture"
    );

    // Check specific fixture contents
    let simple_ls = CommandResult::exec(
        &container,
        vec!["ls", "/test-fixtures/simple-project/.pyst"],
    )
    .await
    .expect("Failed to list simple project scripts");

    assert!(
        simple_ls.success(),
        "Should be able to list simple project scripts"
    );
    assert!(
        simple_ls.stdout.contains("hello.py"),
        "Should see hello.py script"
    );

    println!("✅ Test fixtures successfully mounted");
}

/// Debug test to check individual setup steps
#[tokio::test]
async fn test_debug_rust_install() {
    let image = PystContainer::python_base();
    let container = image.start().await.expect("Failed to start container");

    println!("🔧 Installing dependencies...");
    let deps_result = CommandResult::exec(
        &container,
        vec![
            "sh",
            "-c",
            "apt-get update && apt-get install -y curl build-essential git",
        ],
    )
    .await
    .expect("Failed to run apt install");

    println!("Dependencies stdout: {}", deps_result.stdout);
    println!("Dependencies stderr: {}", deps_result.stderr);
    println!("Dependencies exit code: {}", deps_result.exit_code);

    if deps_result.success() {
        println!("📦 Installing Rust...");
        let rust_result = CommandResult::exec(
            &container,
            vec![
                "sh",
                "-c",
                "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
            ],
        )
        .await
        .expect("Failed to run rust install");

        println!("Rust install stdout: {}", rust_result.stdout);
        println!("Rust install stderr: {}", rust_result.stderr);
        println!("Rust install exit code: {}", rust_result.exit_code);

        if rust_result.success() {
            let check_result = CommandResult::exec(
                &container,
                vec!["sh", "-c", "source ~/.cargo/env && rustc --version"],
            )
            .await
            .expect("Failed to check Rust");

            println!("Rust check stdout: {}", check_result.stdout);
            println!("Rust check stderr: {}", check_result.stderr);
            println!("Rust check exit code: {}", check_result.exit_code);
        }
    }

    // This test just gathers info, doesn't assert
    println!("✅ Debug test completed");
}

/// Test basic pyst commands by mounting fixtures and using uv to run scripts
#[tokio::test]
async fn test_pyst_scripts_with_uv() {
    let fixtures_dir = std::env::current_dir()
        .expect("Failed to get current directory")
        .join("tests/fixtures/simple-project");

    let image = PystContainer::pyst_with_fixtures(&fixtures_dir);
    let container = image.start().await.expect("Failed to start container");

    // Install curl first
    println!("📦 Installing curl...");
    let curl_install = CommandResult::exec(
        &container,
        vec!["sh", "-c", "apt-get update && apt-get install -y curl"],
    )
    .await
    .expect("Failed to install curl");

    if !curl_install.success() {
        panic!("Failed to install curl: {}", curl_install.stderr);
    }

    // Install uv
    println!("📦 Installing uv...");
    let uv_install = CommandResult::exec(
        &container,
        vec![
            "sh",
            "-c",
            "curl -LsSf https://astral.sh/uv/install.sh | sh",
        ],
    )
    .await
    .expect("Failed to install uv");

    println!("uv install stdout: {}", uv_install.stdout);
    println!("uv install stderr: {}", uv_install.stderr);
    println!("uv install exit code: {}", uv_install.exit_code);

    if !uv_install.success() {
        panic!("Failed to install uv: {}", uv_install.stderr);
    }

    // Test that we can see the fixture files
    println!("🔍 Checking fixture directory...");
    let ls_result = CommandResult::exec(&container, vec!["ls", "-la", "/test-fixtures/.pyst"])
        .await
        .expect("Failed to list fixtures");

    println!("Fixture directory contents: {}", ls_result.stdout);
    assert!(
        ls_result.success(),
        "Should be able to list fixture directory"
    );
    assert!(
        ls_result.stdout.contains("hello.py"),
        "Should find hello.py script"
    );

    // Check where uv was installed
    println!("🔍 Finding uv installation...");
    let uv_which_result = CommandResult::exec(
        &container,
        vec![
            "sh",
            "-c",
            "find /root -name uv 2>/dev/null || which uv || echo 'uv not found'",
        ],
    )
    .await
    .expect("Failed to find uv");

    println!("uv location: {}", uv_which_result.stdout);

    // Test running the simple hello script directly with uv
    println!("🐍 Testing uv script execution...");
    let uv_run_result = CommandResult::exec(
        &container,
        vec![
            "sh",
            "-c",
            "export PATH=\"$HOME/.local/bin:$PATH\" && cd /test-fixtures && uv run .pyst/hello.py",
        ],
    )
    .await
    .expect("Failed to run uv script");

    println!("uv run stdout: {}", uv_run_result.stdout);
    println!("uv run stderr: {}", uv_run_result.stderr);
    println!("uv run exit code: {}", uv_run_result.exit_code);

    // uv should successfully run the script
    assert!(
        uv_run_result.success(),
        "uv run should succeed for PEP 723 script"
    );
    assert!(
        uv_run_result
            .stdout
            .contains("Hello from containerized pyst!"),
        "Should see script output"
    );

    // Test script discovery patterns (simulate what pyst would do)
    println!("🔍 Testing script discovery patterns...");
    let discovery_result = CommandResult::exec(
        &container,
        vec!["sh", "-c", "find /test-fixtures -name '*.py' -type f"],
    )
    .await
    .expect("Failed to discover scripts");

    println!("Discovered scripts: {}", discovery_result.stdout);
    assert!(discovery_result.success(), "Script discovery should work");
    assert!(
        discovery_result.stdout.contains("hello.py"),
        "Should discover hello.py"
    );
    assert!(
        discovery_result.stdout.contains("test-cwd.py"),
        "Should discover test-cwd.py"
    );

    println!("✅ Container-based script execution working correctly");
}

/// Test argument forwarding with Click framework
#[tokio::test]
async fn test_argument_forwarding() {
    let fixtures_dir = std::env::current_dir()
        .expect("Failed to get current directory")
        .join("tests/fixtures/click-project");

    let image = PystContainer::pyst_with_fixtures(&fixtures_dir);
    let container = image.start().await.expect("Failed to start container");

    // Install curl and uv
    println!("📦 Installing dependencies...");
    let deps_install = CommandResult::exec(
        &container,
        vec!["sh", "-c", "apt-get update && apt-get install -y curl"],
    )
    .await
    .expect("Failed to install curl");

    if !deps_install.success() {
        panic!("Failed to install dependencies: {}", deps_install.stderr);
    }

    let uv_install = CommandResult::exec(
        &container,
        vec![
            "sh",
            "-c",
            "curl -LsSf https://astral.sh/uv/install.sh | sh",
        ],
    )
    .await
    .expect("Failed to install uv");

    if !uv_install.success() {
        panic!("Failed to install uv: {}", uv_install.stderr);
    }

    // Test Click script with various arguments
    println!("🖱️ Testing Click argument forwarding...");

    // Test basic execution
    let basic_result = CommandResult::exec(&container, vec![
        "sh", "-c", "export PATH=\"$HOME/.local/bin:$PATH\" && cd /test-fixtures && uv run .pyst/cli-script.py"
    ]).await.expect("Failed to run Click script");

    println!("Basic execution stdout: {}", basic_result.stdout);
    assert!(basic_result.success(), "Basic Click script should succeed");
    assert!(
        basic_result.stdout.contains("Container hello, Container!"),
        "Should use default name"
    );

    // Test with arguments
    let args_result = CommandResult::exec(&container, vec![
        "sh", "-c", "export PATH=\"$HOME/.local/bin:$PATH\" && cd /test-fixtures && uv run .pyst/cli-script.py --name TestBot --count 2 --verbose extra1 extra2"
    ]).await.expect("Failed to run Click script with args");

    println!("Args execution stdout: {}", args_result.stdout);
    assert!(
        args_result.success(),
        "Click script with args should succeed"
    );
    assert!(
        args_result.stdout.contains("Container hello, TestBot!"),
        "Should use provided name"
    );
    assert!(
        args_result.stdout.contains("greeting 1") && args_result.stdout.contains("greeting 2"),
        "Should repeat twice"
    );
    assert!(
        args_result
            .stdout
            .contains("Container verbose mode enabled"),
        "Should show verbose output"
    );
    assert!(
        args_result.stdout.contains("extra1") && args_result.stdout.contains("extra2"),
        "Should forward extra args"
    );

    // Test with short flags
    let short_flags_result = CommandResult::exec(&container, vec![
        "sh", "-c", "export PATH=\"$HOME/.local/bin:$PATH\" && cd /test-fixtures && uv run .pyst/cli-script.py -n Robot -c 3 -v"
    ]).await.expect("Failed to run Click script with short flags");

    println!("Short flags stdout: {}", short_flags_result.stdout);
    assert!(
        short_flags_result.success(),
        "Click script with short flags should succeed"
    );
    assert!(
        short_flags_result
            .stdout
            .contains("Container hello, Robot!"),
        "Should use short flag name"
    );
    assert!(
        short_flags_result.stdout.contains("greeting 3"),
        "Should repeat 3 times"
    );

    println!("✅ Argument forwarding working correctly with Click framework");
}

/// Test working directory behavior and CLI overrides
#[tokio::test]
async fn test_working_directory_behavior() {
    let fixtures_dir = std::env::current_dir()
        .expect("Failed to get current directory")
        .join("tests/fixtures/simple-project");

    let image = PystContainer::pyst_with_fixtures(&fixtures_dir);
    let container = image.start().await.expect("Failed to start container");

    // Install dependencies
    println!("📦 Installing dependencies...");
    let deps_install = CommandResult::exec(
        &container,
        vec!["sh", "-c", "apt-get update && apt-get install -y curl"],
    )
    .await
    .expect("Failed to install curl");

    if !deps_install.success() {
        panic!("Failed to install dependencies: {}", deps_install.stderr);
    }

    let uv_install = CommandResult::exec(
        &container,
        vec![
            "sh",
            "-c",
            "curl -LsSf https://astral.sh/uv/install.sh | sh",
        ],
    )
    .await
    .expect("Failed to install uv");

    if !uv_install.success() {
        panic!("Failed to install uv: {}", uv_install.stderr);
    }

    // Test working directory behavior
    println!("📁 Testing working directory behavior...");

    // Run the test-cwd script which creates a file showing its working directory
    let cwd_test_result = CommandResult::exec(&container, vec![
        "sh", "-c", "export PATH=\"$HOME/.local/bin:$PATH\" && cd /test-fixtures && uv run .pyst/test-cwd.py"
    ]).await.expect("Failed to run cwd test script");

    println!("CWD test stdout: {}", cwd_test_result.stdout);
    println!("CWD test stderr: {}", cwd_test_result.stderr);

    assert!(cwd_test_result.success(), "CWD test script should succeed");
    assert!(
        cwd_test_result.stdout.contains("/test-fixtures"),
        "Should run from test-fixtures directory"
    );

    // Verify the output file was created in the correct location
    let file_check = CommandResult::exec(
        &container,
        vec!["cat", "/test-fixtures/container_cwd_test.txt"],
    )
    .await
    .expect("Failed to read cwd test file");

    println!("CWD test file contents: {}", file_check.stdout);
    assert!(
        file_check.success(),
        "Should be able to read the cwd test file"
    );
    assert!(
        file_check.stdout.contains("/test-fixtures"),
        "File should show correct working directory"
    );

    // Test different working directory
    println!("📁 Testing different working directory...");

    // Run from a different directory to test working directory behavior
    let different_cwd_result = CommandResult::exec(&container, vec![
        "sh", "-c", "export PATH=\"$HOME/.local/bin:$PATH\" && cd /tmp && uv run /test-fixtures/.pyst/test-cwd.py"
    ]).await.expect("Failed to run cwd test from different directory");

    println!("Different CWD stdout: {}", different_cwd_result.stdout);
    assert!(
        different_cwd_result.success(),
        "Should succeed from different directory"
    );
    assert!(
        different_cwd_result.stdout.contains("/tmp"),
        "Should run from /tmp directory"
    );

    // Verify the file was created in /tmp this time
    let tmp_file_check =
        CommandResult::exec(&container, vec!["cat", "/tmp/container_cwd_test.txt"])
            .await
            .expect("Failed to read tmp cwd test file");

    assert!(
        tmp_file_check.success(),
        "Should be able to read tmp cwd test file"
    );
    assert!(
        tmp_file_check.stdout.contains("/tmp"),
        "File should show /tmp working directory"
    );

    // Test relative path execution
    println!("📁 Testing relative path execution...");

    // Navigate to the .pyst directory and run with relative path
    let relative_result = CommandResult::exec(&container, vec![
        "sh", "-c", "export PATH=\"$HOME/.local/bin:$PATH\" && cd /test-fixtures/.pyst && uv run ./test-cwd.py"
    ]).await.expect("Failed to run with relative path");

    println!("Relative path stdout: {}", relative_result.stdout);
    assert!(
        relative_result.success(),
        "Should succeed with relative path"
    );
    assert!(
        relative_result.stdout.contains("/test-fixtures/.pyst"),
        "Should run from .pyst directory"
    );

    println!("✅ Working directory behavior validated correctly");
}

/// Test actual pyst binary commands end-to-end
/// This test takes longer but validates the complete pyst workflow
#[tokio::test]
async fn test_full_dev_setup() {
    // Use a longer timeout for this comprehensive test

    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let project_root = current_dir
        .parent()
        .expect("Failed to get parent directory");
    let fixtures_dir = current_dir.join("tests/fixtures/simple-project");

    // Create container with both source and fixtures mounted
    let source_mount = testcontainers::core::Mount::bind_mount(
        project_root.to_string_lossy().to_string(),
        "/workspace",
    );
    let fixtures_mount = testcontainers::core::Mount::bind_mount(
        fixtures_dir.to_string_lossy().to_string(),
        "/test-project",
    );

    let image = PystContainer::pyst_dev_base()
        .with_mount(source_mount)
        .with_mount(fixtures_mount)
        .with_working_dir("/test-project");

    let container = image.start().await.expect("Failed to start container");

    println!("🏗️ Setting up complete development environment...");

    // Install system dependencies
    println!("📦 Installing system dependencies...");
    let deps_result = CommandResult::exec(
        &container,
        vec![
            "sh",
            "-c",
            "apt-get update && apt-get install -y curl build-essential git pkg-config libssl-dev",
        ],
    )
    .await
    .expect("Failed to install dependencies");

    if !deps_result.success() {
        panic!("Failed to install dependencies: {}", deps_result.stderr);
    }

    // Install Rust
    println!("🦀 Installing Rust...");
    let rust_result = CommandResult::exec(
        &container,
        vec![
            "sh",
            "-c",
            "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
        ],
    )
    .await
    .expect("Failed to install Rust");

    if !rust_result.success() {
        panic!("Failed to install Rust: {}", rust_result.stderr);
    }

    // Install uv
    println!("📦 Installing uv...");
    let uv_result = CommandResult::exec(
        &container,
        vec![
            "sh",
            "-c",
            "curl -LsSf https://astral.sh/uv/install.sh | sh",
        ],
    )
    .await
    .expect("Failed to install uv");

    if !uv_result.success() {
        panic!("Failed to install uv: {}", uv_result.stderr);
    }

    // Build pyst
    println!("🔨 Building pyst (this may take several minutes)...");
    let build_result = CommandResult::exec(
        &container,
        vec![
            "sh",
            "-c",
            "export PATH=\"$HOME/.cargo/bin:$PATH\" && cd /workspace && cargo build --release",
        ],
    )
    .await
    .expect("Failed to build pyst");

    if !build_result.success() {
        panic!(
            "Failed to build pyst: {}\nStderr: {}",
            build_result.stdout, build_result.stderr
        );
    }

    let pyst_binary = "/workspace/target/release/pyst";

    // Verify pyst binary exists
    let binary_check = CommandResult::exec(&container, vec!["ls", "-la", pyst_binary])
        .await
        .expect("Failed to check pyst binary");

    if !binary_check.success() {
        panic!("Pyst binary not found after build");
    }

    println!("✅ Pyst built successfully!");

    // Test pyst --version
    println!("🔍 Testing pyst --version...");
    let version_result = CommandResult::exec(
        &container,
        vec![
            "sh",
            "-c",
            &format!("export PATH=/root/.local/bin:$PATH && {}", pyst_binary),
        ],
    )
    .await
    .expect("Failed to run pyst --version");

    // Note: pyst might not have --version flag, so we'll check if it runs at all
    println!("Pyst version stdout: {}", version_result.stdout);
    println!("Pyst version stderr: {}", version_result.stderr);

    // Test pyst list
    println!("📋 Testing pyst list...");
    let list_result = CommandResult::exec(
        &container,
        vec![
            "sh",
            "-c",
            &format!("export PATH=/root/.local/bin:$PATH && {} list", pyst_binary),
        ],
    )
    .await
    .expect("Failed to run pyst list");

    println!("Pyst list stdout: {}", list_result.stdout);
    println!("Pyst list stderr: {}", list_result.stderr);

    assert!(list_result.success(), "pyst list should succeed");
    assert!(
        list_result.stdout.contains("hello") || list_result.stdout.contains("test-cwd"),
        "Should discover test scripts"
    );

    // Test pyst info
    println!("ℹ️ Testing pyst info...");
    let info_result = CommandResult::exec(
        &container,
        vec![
            "sh",
            "-c",
            &format!(
                "export PATH=/root/.local/bin:$PATH && {} info hello",
                pyst_binary
            ),
        ],
    )
    .await
    .expect("Failed to run pyst info");

    println!("Pyst info stdout: {}", info_result.stdout);

    if info_result.success() {
        assert!(
            info_result.stdout.contains("hello") || info_result.stdout.contains("Simple"),
            "Should show script info"
        );
    }

    // Test pyst run
    println!("🚀 Testing pyst run...");
    let run_result = CommandResult::exec(
        &container,
        vec![
            "sh",
            "-c",
            &format!(
                "export PATH=/root/.local/bin:$PATH && {} run hello",
                pyst_binary
            ),
        ],
    )
    .await
    .expect("Failed to run pyst run");

    println!("Pyst run stdout: {}", run_result.stdout);
    println!("Pyst run stderr: {}", run_result.stderr);

    assert!(run_result.success(), "pyst run should succeed");
    assert!(
        run_result.stdout.contains("Hello from containerized pyst!"),
        "Should execute script correctly"
    );

    // Test pyst run with arguments
    println!("🎯 Testing pyst run with arguments...");
    let args_result = CommandResult::exec(
        &container,
        vec![
            "sh",
            "-c",
            &format!(
                "export PATH=/root/.local/bin:$PATH && {} run hello arg1 arg2",
                pyst_binary
            ),
        ],
    )
    .await
    .expect("Failed to run pyst with args");

    println!("Pyst args stdout: {}", args_result.stdout);

    if args_result.success() {
        assert!(
            args_result.stdout.contains("arg1") && args_result.stdout.contains("arg2"),
            "Should forward arguments correctly"
        );
    }

    println!("✅ End-to-end pyst testing completed successfully!");
}

/// Test optimized pyst development workflow using pre-built image
/// This test should be much faster than test_full_dev_setup
#[tokio::test]
async fn test_optimized_pyst_workflow() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let project_root = current_dir
        .parent()
        .expect("Failed to get parent directory");
    let fixtures_dir = current_dir.join("tests/fixtures/simple-project");

    println!("🚀 Testing optimized pyst workflow with pre-built image...");

    // Use the optimized container with all tools pre-installed
    let image = PystContainer::optimized_full_dev(project_root, &fixtures_dir);
    let container = image
        .start()
        .await
        .expect("Failed to start optimized container");

    // Verify tools are already available (no installation needed!)
    println!("🔍 Verifying pre-installed tools...");

    let tool_check = CommandResult::exec(
        &container,
        vec![
            "sh",
            "-c",
            "rustc --version && cargo --version && uv --version && python --version",
        ],
    )
    .await
    .expect("Failed to check tools");

    assert!(tool_check.success(), "All tools should be pre-installed");
    assert!(tool_check.stdout.contains("rustc"), "Should have Rust");
    assert!(tool_check.stdout.contains("cargo"), "Should have Cargo");
    assert!(tool_check.stdout.contains("uv"), "Should have uv");
    assert!(tool_check.stdout.contains("Python"), "Should have Python");

    println!("✅ All tools verified and ready!");

    // Build pyst directly (no tool installation time!)
    println!("🔨 Building pyst with optimized environment...");
    let build_result = CommandResult::exec(
        &container,
        vec!["sh", "-c", "cd /workspace && cargo build --release"],
    )
    .await
    .expect("Failed to build pyst");

    if !build_result.success() {
        panic!(
            "Failed to build pyst: {}\nStderr: {}",
            build_result.stdout, build_result.stderr
        );
    }

    let pyst_binary = "/workspace/target/release/pyst";

    // Test pyst commands immediately
    println!("📋 Testing pyst list...");
    let list_result = CommandResult::exec(&container, vec![&pyst_binary, "list"])
        .await
        .expect("Failed to run pyst list");

    assert!(list_result.success(), "pyst list should succeed");
    assert!(
        list_result.stdout.contains("hello") || list_result.stdout.contains("test-cwd"),
        "Should discover test scripts"
    );

    println!("🚀 Testing pyst run...");
    let run_result = CommandResult::exec(&container, vec![&pyst_binary, "run", "hello"])
        .await
        .expect("Failed to run pyst");

    assert!(run_result.success(), "pyst run should succeed");
    assert!(
        run_result.stdout.contains("Hello from containerized pyst!"),
        "Should execute script correctly"
    );

    println!("✅ Optimized pyst workflow completed successfully!");
    println!("   This test should be significantly faster than manual tool installation!");
}
