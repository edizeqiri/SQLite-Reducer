# Build stage
FROM rust:latest as builder

WORKDIR /usr/src/app
COPY . .

# Build the project
RUN cargo build --release

# Final stage
FROM theosotr/sqlite3-test:latest

USER root
# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/reducer /usr/bin/reducer

# Copy the queries and the test script
COPY queries queries
COPY src/resources/native.sh native.sh
COPY src/resources/full_run.sh full_run.sh
RUN mkdir -p /output
RUN mkdir -p /output/logs
ENV SQL_NUMBER=1

ENV RUST_LOG=warn

