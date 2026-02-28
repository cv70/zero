# syntax=docker/dockerfile:1
FROM rust:1.70-slim AS builder
WORKDIR /workspace
COPY . .
RUN cargo build --release --workspace

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /workspace/target/release/ /usr/local/bin/
COPY deploy/docker/run-module.sh /usr/local/bin/run-module
RUN chmod +x /usr/local/bin/run-module
ENV RUST_LOG=info
ENTRYPOINT ["/usr/local/bin/run-module"]
