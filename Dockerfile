# Stage 1: Build the Flutter Web application
FROM ghcr.io/cirruslabs/flutter:stable AS build

# Configurar directorio de trabajo
WORKDIR /app

# Copiar archivos de dependencias para caching
COPY pubspec.yaml pubspec.lock ./
RUN flutter pub get

# Copiar el código del proyecto (optimizado por .dockerignore)
COPY . .

# Argumento de construcción para definir el endpoint del API backend
ARG BACKEND_URL
RUN flutter build web --release --dart-define=BACKEND_URL=${BACKEND_URL}

# Stage 2: Servir los archivos estáticos usando Nginx
FROM nginx:alpine
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/build/web /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
