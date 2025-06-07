# ---------- Base image ----------
# Use the official Rust image (Debian Bullseye + Rust toolchain)
FROM theosotr/sqlite3-test:latest

USER root

# ---------- Install OS dependencies ----------
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      build-essential \
      cmake \
      pkg-config \
      libssl-dev \
      curl \
      git \
      openssh-server \
 && rm -rf /var/lib/apt/lists/*

 # Install Python dependencies
 # Install Python and pip
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 \
    python3-pip \
    python3-venv \
 && rm -rf /var/lib/apt/lists/*

RUN pip install --upgrade pip

# Install Jupyter and ipykernel (for the Jupyter kernel)
RUN pip install --upgrade --quiet jupyter ipykernel

# Register the current Python environment as a Jupyter kernel
RUN python3 -m ipykernel install --user --name=mykernel --display-name "Python 3 (mycontainer)"

# ---------- Install Rust toolchain ----------
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y && \
    . "$HOME/.cargo/env"

# ---------- Configure SSH for root login ----------
# Create SSH run directory, set root password, permit root login
RUN mkdir -p /var/run/sshd \
 && echo 'root:root' | chpasswd \
 && sed -i 's/^#\?PermitRootLogin .*/PermitRootLogin yes/' /etc/ssh/sshd_config \
 && sed -i 's/^#\?PasswordAuthentication .*/PasswordAuthentication yes/' /etc/ssh/sshd_config

# ---------- Create a workspace directory ----------
RUN mkdir /workspace

RUN sudo apt update
RUN sudo apt install dos2unix
COPY requirements.txt .
RUN pip install -r requirements.txt

# ---------- Environment and working dir ----------
ENV CARGO_HOME=/home/root/.cargo
ENV RUSTUP_HOME=/home/root/.rustup
ENV PATH="$CARGO_HOME/bin:${PATH}"
ENV TEST_CASE_LOCATION=/ops/query.sql

ENV CARGO_TARGET_DIR=/target
ENV RUST_LOG=info

WORKDIR /workspace

# ---------- Expose SSH port and start SSHD by default ----------
EXPOSE 22
#RUN "/usr/sbin/sshd -D"

# Expose the port Jupyter will run on
EXPOSE 8888

# Set the default command to run the start-up script
COPY start.sh /start.sh
RUN chmod +x /start.sh
CMD ["/start.sh"]
