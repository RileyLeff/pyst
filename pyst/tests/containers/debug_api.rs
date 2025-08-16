use testcontainers::{core::ExecCommand, runners::AsyncRunner, GenericImage, ImageExt};

#[tokio::test]
async fn debug_container_api() {
    // Use a container that stays running with a long-running process
    let image = GenericImage::new("python", "3.11-slim").with_cmd(vec!["sleep", "infinity"]); // Keep container running

    let container = image.start().await.expect("Failed to start container");

    println!("Container started successfully");

    // Test simple command first
    let whoami_command = ExecCommand::new(vec!["whoami"]);
    let result = container.exec(whoami_command).await;

    match result {
        Ok(mut exec_result) => {
            println!("✅ Whoami command executed");

            let stdout = exec_result
                .stdout_to_vec()
                .await
                .expect("Failed to get stdout");
            let stdout_str = String::from_utf8_lossy(&stdout);
            println!("Whoami output: {}", stdout_str.trim());

            if let Some(exit_code) = exec_result
                .exit_code()
                .await
                .expect("Failed to get exit code")
            {
                println!("Whoami exit code: {}", exit_code);
            }
        }
        Err(e) => {
            println!("❌ Whoami command failed: {:?}", e);
        }
    }

    // Test Python version command with output capture
    let exec_command = ExecCommand::new(vec!["python", "--version"]);
    let result = container.exec(exec_command).await;

    match result {
        Ok(mut exec_result) => {
            println!("✅ Python version command executed");

            // Capture stdout output
            let stdout = exec_result
                .stdout_to_vec()
                .await
                .expect("Failed to get stdout");
            let stderr = exec_result
                .stderr_to_vec()
                .await
                .expect("Failed to get stderr");

            let stdout_str = String::from_utf8_lossy(&stdout);
            let stderr_str = String::from_utf8_lossy(&stderr);

            println!("Python version stdout: '{}'", stdout_str.trim());
            println!("Python version stderr: '{}'", stderr_str.trim());

            // Check exit code
            if let Some(exit_code) = exec_result
                .exit_code()
                .await
                .expect("Failed to get exit code")
            {
                println!("Python version exit code: {}", exit_code);
            }
        }
        Err(e) => {
            println!("❌ Python version command failed: {:?}", e);
        }
    }

    // Test ls command
    let ls_command = ExecCommand::new(vec!["ls", "-la", "/"]);
    let ls_result = container.exec(ls_command).await;

    match ls_result {
        Ok(mut exec_result) => {
            println!("✅ LS command executed");

            let stdout = exec_result
                .stdout_to_vec()
                .await
                .expect("Failed to get stdout");
            let stdout_str = String::from_utf8_lossy(&stdout);
            println!(
                "LS output (first 200 chars): {}",
                stdout_str.chars().take(200).collect::<String>()
            );

            if let Some(exit_code) = exec_result
                .exit_code()
                .await
                .expect("Failed to get exit code")
            {
                println!("LS exit code: {}", exit_code);
            }
        }
        Err(e) => {
            println!("❌ LS command failed: {:?}", e);
        }
    }

    println!("✅ All tests completed - container will auto-cleanup");
}
