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
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Rust components needed for ESP32
RUN rustup component add rust-src

# Install espup and set up ESP Rust toolchain
RUN cargo install espup && \
    espup install && \
    cargo install ldproxy

# Source ESP environment variables in all shells
ENV LIBCLANG_PATH="/root/.rustup/toolchains/esp/xtensa-esp32-elf-clang/esp-16.0.0-20230516/esp-clang/lib"
ENV PATH="/root/.rustup/toolchains/esp/xtensa-esp32-elf/esp-13.2.0_20230928/xtensa-esp32-elf/bin:${PATH}"
ENV IDF_TOOLS_PATH="/root/.espressif"

# Set working directory
WORKDIR /project

# Set default WiFi credentials (can be overridden at build time)
ENV WIFI_SSID="your_wifi_ssid"
ENV WIFI_PASS="your_wifi_password"

# Copy the project files
COPY . .

# Build the project
# Note: WiFi credentials can be passed as build arguments
ARG WIFI_SSID
ARG WIFI_PASS
ENV WIFI_SSID=${WIFI_SSID:-your_wifi_ssid}
ENV WIFI_PASS=${WIFI_PASS:-your_wifi_password}

# The default command builds the release binary
# The output will be at /project/target/xtensa-esp32-espidf/release/wre
CMD ["/bin/bash", "-c", "source $HOME/export-esp.sh && cargo build --release && echo '\n=== Build Complete ===' && echo 'Release binary location:' && ls -lh target/xtensa-esp32-espidf/release/wre"]
