# Dockerfile for building WRE (Wireless Rotary Encoder) firmware for ESP32
# This provides a reproducible build environment with all dependencies pre-installed

FROM ubuntu:22.04

# Prevent interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && apt-get install -y \
    git \
    curl \
    gcc \
    clang \
    ninja-build \
    cmake \
    libuv1-dev \
    libssl-dev \
    libpython3-dev \
    python3 \
    python3-pip \
    python3-venv \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Rust components needed for ESP32
RUN rustup component add rust-src

# Install espup
RUN cargo install espup

# Install ldproxy
RUN cargo install ldproxy

# Set up ESP Rust toolchain
# This step is run when the container starts to avoid GitHub API rate limits during image build
# You can also set this up manually with: docker run ... /bin/bash -c "espup install && cargo build --release"

# Set working directory
WORKDIR /project

# Set default WiFi credentials (can be overridden at build/run time)
ENV WIFI_SSID="your_wifi_ssid"
ENV WIFI_PASS="your_wifi_password"

# Build the project when container runs
# The default command sets up ESP toolchain if needed, then builds
CMD ["/bin/bash", "-c", "\
    echo 'Setting up ESP Rust toolchain...' && \
    (test -f $HOME/export-esp.sh || espup install) && \
    source $HOME/export-esp.sh && \
    echo 'Building WRE firmware...' && \
    cargo build --release && \
    echo '' && \
    echo '=== Build Complete ===' && \
    echo 'Release binary location:' && \
    ls -lh target/xtensa-esp32-espidf/release/wre && \
    echo '' && \
    echo 'To flash: espflash flash --monitor target/xtensa-esp32-espidf/release/wre'\
"]
