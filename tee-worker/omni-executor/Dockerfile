# syntax=docker/dockerfile:1

FROM litentry/litentry-tee-dev:latest AS builder
WORKDIR /home/ubuntu/tee-worker/omni-executor
COPY . /home/ubuntu
RUN cargo build --release

FROM ubuntu:22.04 AS executor-worker
COPY --from=local-builder:latest /home/ubuntu/tee-worker/omni-executor/target/release/executor-worker /usr/local/bin/executor-worker

RUN \
  apt-get update && \
  apt-get install -y ca-certificates && \
  apt-get clean

CMD ["executor-worker"]