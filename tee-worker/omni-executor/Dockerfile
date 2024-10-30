FROM rust:1.82 AS builder
WORKDIR /usr/src/omni-executor
COPY . .
RUN cargo build --release

FROM ubuntu:22.04
COPY --from=builder /usr/src/omni-executor/target/release/executor-worker /usr/local/bin/executor-worker

RUN \
  apt-get update && \
  apt-get install -y ca-certificates && \
  apt-get clean

CMD ["executor-worker"]