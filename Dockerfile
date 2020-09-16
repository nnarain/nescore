FROM rust:latest

RUN apt-get update \
    && apt-get install -y linux-base linux-perf
    && apt-get install -y cmake

RUN cargo install flamegraph

CMD ["sleep", "infinity"]
