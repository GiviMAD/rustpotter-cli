# Prepare vendor on hosted platform
FROM --platform=$BUILDPLATFORM rust:buster as rust_vendor
WORKDIR /code
RUN cargo init
COPY src /code/src
COPY Cargo.* /code/
RUN mkdir -p /code/.cargo \
  && cargo vendor > /code/.cargo/config
# build
FROM rust:buster
COPY src /code/src
COPY Cargo.* /code/
COPY build.rs /code/
COPY --from=rust_vendor /code/.cargo /code/.cargo
COPY --from=rust_vendor /code/vendor /code/vendor
WORKDIR code
RUN rustup component add rustfmt && \
    cargo build --release --features dist --offline && \
    mkdir -p output && \
    cp target/release/rustpotter-cli output/rustpotter-cli_debian_$(arch) && \
    rm -rf target

