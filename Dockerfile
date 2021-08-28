FROM rust:1.92-slim AS builder
WORKDIR /usr/src/dj_ferris
COPY . .
RUN apt-get update && apt-get install opus-tools cmake -y
RUN cargo install --path .

FROM debian:stable-slim
COPY --from=builder /usr/local/cargo/bin/dj_ferris /usr/local/bin/dj_ferris
RUN apt-get update && apt-get install ca-certificates -y

CMD ["dj_ferris"]
