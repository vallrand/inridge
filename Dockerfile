FROM rust:latest as builder

RUN apt update && apt upgrade -y
RUN apt install -y g++-mingw-w64-x86-64

RUN rustup target add x86_64-pc-windows-gnu
RUN rustup toolchain install stable-x86_64-pc-windows-gnu

RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-bindgen-cli

WORKDIR /app
COPY Cargo.* .
COPY src ./src

FROM builder as builder-windows

RUN cargo build --release --target x86_64-pc-windows-gnu

FROM builder as builder-wasm

RUN cargo build --profile wasm-release --target wasm32-unknown-unknown
RUN wasm-bindgen --out-dir ./output/ --target web ./target/wasm32-unknown-unknown/wasm-release/*.wasm

FROM scratch as release-windows
COPY --from=builder-windows /app/target/x86_64-pc-windows-gnu/release/*.exe .

FROM scratch as release-wasm
COPY --from=builder-wasm /app/output .