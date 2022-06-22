FROM rust:1.61 as builder

WORKDIR /usr/src/raw
COPY . .
RUN cargo install --path .


FROM debian:buster-slim
RUN apt-get update && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/raw /usr/local/bin/raw

CMD ["raw"]