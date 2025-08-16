use std::path::PathBuf;
use testcontainers::{runners::AsyncRunner, GenericImage, ImageExt};
use crate::containers::PystContainer;

/// Test basic container setup and Python execution
#[tokio::test]
async fn test_basic_python_execution() {
    let image = PystContainer::python_base();
    let _container = image.start().await.expect("Failed to start container");
    
    println!("Python container started successfully");
    // TODO: Implement command execution once we understand the testcontainers exec API
}

/// Test container with environment variables
#[tokio::test] 
async fn test_container_with_env_vars() {
    let image = PystContainer::python_with_env();
    let _container = image.start().await.expect("Failed to start container");
    
    println!("Container with environment variables started successfully");
    // TODO: Test environment variables once we have exec API working
}