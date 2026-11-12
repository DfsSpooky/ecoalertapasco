# Usar imagen base oficial de Python
FROM python:3.12-slim

# Variables de entorno
ENV PYTHONDONTWRITEBYTECODE=1
ENV PYTHONUNBUFFERED=1

# Directorio de trabajo
WORKDIR /app

# Instalar dependencias del sistema
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Instalar dependencias del proyecto
COPY requirements.txt /app/
RUN pip install --no-cache-dir -r requirements.txt

# Copiar el código del proyecto
COPY . /app/

# Asegurar directorios persistentes de medios y archivos estáticos
RUN mkdir -p /app/media/uploads /app/staticfiles

# Exponer puerto
EXPOSE 8000

# Comando de inicio ASGI para producción (HTTP + WebSockets en tiempo real con Uvicorn + uvloop):
# - Ejecuta migraciones sin borrar datos reales
# - Inicializa los puntos municipales de acopio únicamente si la tabla está vacía (--if-empty)
# - Levanta Uvicorn soportando cabeceras de Reverse Proxy HTTPS/WSS (--proxy-headers)
CMD ["sh", "-c", "python manage.py migrate --noinput && python manage.py collectstatic --noinput && python manage.py seed_waste_points --if-empty && python manage.py seed_collection_routes --if-empty && python manage.py ensure_admin && uvicorn ecoalerta_backend.asgi:application --host 0.0.0.0 --port 8000 --proxy-headers --forwarded-allow-ips='*' --ws-ping-interval 25 --ws-ping-timeout 30"]
