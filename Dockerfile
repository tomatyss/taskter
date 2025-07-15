FROM rust:1.88.0 as builder
WORKDIR /usr/src/taskter
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/taskter/target/release/taskter /usr/local/bin/taskter
WORKDIR /workspace
ENTRYPOINT ["taskter"]
CMD ["--help"]
