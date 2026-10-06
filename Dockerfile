# Stage 1: Build the Flutter Web application
FROM debian:bookworm-slim AS build

# Instalar dependencias necesarias
RUN apt-get update && apt-get install -y \
    curl \
    git \
    unzip \
    xz-utils \
    zip \
    libglu1-mesa \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Descargar e instalar Flutter SDK stable
RUN git clone https://github.com/flutter/flutter.git -b stable --depth 1 /usr/local/flutter
ENV PATH="/usr/local/flutter/bin:/usr/local/flutter/bin/cache/dart-sdk/bin:${PATH}"

# Forzar la predescarga de dependencias de Flutter
RUN flutter doctor

# Configurar directorio de trabajo
WORKDIR /app

# Copiar archivos de dependencias para caching
COPY pubspec.yaml pubspec.lock ./
RUN flutter pub get

# Copiar el código del proyecto
COPY . .

# Argumento de construcción para definir el endpoint del API backend
ARG BACKEND_URL
RUN flutter build web --release --dart-define=BACKEND_URL=${BACKEND_URL}

# Stage 2: Servir los archivos estáticos usando Nginx
FROM nginx:alpine
COPY --from=build /app/build/web /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
