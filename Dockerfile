FROM rust:buster as builder
COPY . .
RUN cargo build --bin server --release

FROM gcr.io/distroless/cc
ENV RUST_LOG=debug
COPY --from=builder ./target/release/server ./targit/release/server
CMD ["/target/release/server"]
