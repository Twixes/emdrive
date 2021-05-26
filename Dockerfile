FROM rust:1.52-slim AS builder
WORKDIR /usr/src/metrobaza
COPY . .
RUN cargo install --path .

FROM alpine:3
COPY --from=builder /usr/local/cargo/bin/metrobaza /usr/local/bin/metrobaza
CMD ["metrobaza"]
