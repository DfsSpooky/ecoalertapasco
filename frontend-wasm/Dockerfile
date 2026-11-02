# Stage 1: Compilar Frontend Rust + Leptos a WebAssembly optimizado para producción
FROM rust:1.85-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown && cargo install --locked trunk

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
COPY index.html manifest.json sw.js Trunk.toml* ./
COPY src ./src

ARG BACKEND_URL=""
ENV BACKEND_URL=${BACKEND_URL}

RUN trunk build --release

# Stage 2: Servidor Nginx ligero con compresión Gzip para WebAssembly e inyección de BACKEND_URL
FROM nginx:1.27-alpine

COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY docker-entrypoint.sh /docker-entrypoint-ecoalerta.sh
RUN chmod +x /docker-entrypoint-ecoalerta.sh

EXPOSE 80
ENTRYPOINT ["/docker-entrypoint-ecoalerta.sh"]
