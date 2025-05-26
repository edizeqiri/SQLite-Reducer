# ---------- Base image ----------
# Use the official Rust image (Debian Bullseye + Rust toolchain)
FROM rust:1.67-bullseye

# ---------- Install OS dependencies ----------
# Adjust as needed for your project (e.g. add libssl-dev, pkg-config, etc.)
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      build-essential \
      cmake \
      pkg-config \
      libssl-dev \
      curl \
      git \
 && rm -rf /var/lib/apt/lists/*

# ---------- Add Rust components ----------
# Clippy (linter) and rustfmt (formatter)
RUN rustup component add clippy rustfmt

# ---------- Create a non-root “developer” user ----------
# (so VS Code doesn’t run as root inside the container)
ARG USERNAME=dev
ARG USER_UID=1000
ARG USER_GID=1000
RUN groupadd --gid $USER_GID $USERNAME \
 && useradd --uid $USER_UID --gid $USER_GID -m $USERNAME \
 && mkdir -p /workspace \
 && chown $USERNAME:$USERNAME /workspace

# ---------- Environment and working dir ----------
ENV CARGO_HOME=/home/$USERNAME/.cargo
ENV RUSTUP_HOME=/home/$USERNAME/.rustup
ENV PATH="$CARGO_HOME/bin:${PATH}"

WORKDIR /workspace
USER $USERNAME

# ---------- (Optional) Pre-cache dependencies ----------
# If you have a Cargo.toml, uncomment to pre-fetch crates on build
#COPY /Users/edi/Github/reducer/Cargo.toml /Users/edi/Github/reducer/Cargo.lock ./
#RUN cargo fetch

# ---------- Final hints ----------
# - Ports, volumes, and other settings are configured in devcontainer.json
# - You can extend this by installing tools like `llvm-tools-preview` for wasm, etc.
