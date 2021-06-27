FROM rust:latest as builder
WORKDIR /usr/src/mongodb-test
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/mongodb-test /usr/local/bin/mongodb-test
CMD ["mongodb-test"]