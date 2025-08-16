# Optimized test container for pyst with all tools pre-installed
FROM python:3.11-slim

# Set environment variables
ENV PYTHONUNBUFFERED=1
ENV DEBIAN_FRONTEND=noninteractive
ENV UV_NO_NETWORK=0
ENV PATH="/root/.cargo/bin:/root/.local/bin:${PATH}"

# Install all system dependencies in one layer
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    git \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Install Rust with specific components
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    --default-toolchain stable \
    --profile minimal \
    && /root/.cargo/bin/rustup component add clippy rustfmt

# Install uv
RUN curl -LsSf https://astral.sh/uv/install.sh | sh

# Verify installations and warm up caches
RUN rustc --version && \
    cargo --version && \
    uv --version && \
    python --version

# Pre-create common directories
RUN mkdir -p /workspace /test-fixtures /test-project

# Set working directory
WORKDIR /workspace

# Keep container running
CMD ["sleep", "infinity"]