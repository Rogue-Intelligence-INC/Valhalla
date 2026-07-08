# Valhalla Base — CPU-oriented reproduction image (smoke + suites without GPU)
FROM rust:1.83-bookworm AS builder

WORKDIR /build
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY . .
ARG VALHALLA_GIT_SHA=docker
ENV VALHALLA_GIT_SHA=${VALHALLA_GIT_SHA}
ENV VALHALLA_USE_CPU=1

RUN cargo build -p hub-f64 --release --bin valhalla_base --features patch-lm && \
    cargo build -p hub-f64 --release --bin valhalla_dashboard --features patch-lm

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates python3 python3-pip curl && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /valhalla
COPY --from=builder /build/target/release/valhalla_base /usr/local/bin/
COPY --from=builder /build/target/release/valhalla_dashboard /usr/local/bin/
COPY --from=builder /build/tools /valhalla/tools
COPY --from=builder /build/Makefile /valhalla/
COPY --from=builder /build/REPRODUCTION.md /valhalla/
COPY --from=builder /build/requirements-benchmark.txt /valhalla/

ENV VALHALLA_USE_CPU=1
ENV PATH="/valhalla:${PATH}"

EXPOSE 8787 8765
CMD ["valhalla_base", "serve", "--with-dashboard", "--port", "8787"]
