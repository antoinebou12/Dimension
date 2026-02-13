# Stage 1: build all WASM demos and populate website/
FROM rust:1-bookworm AS builder
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-pack wasm-bindgen-cli just
WORKDIR /build
COPY . .
RUN just website-build

# Stage 2: serve static site with nginx
FROM nginx:alpine
COPY --from=builder /build/website /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
