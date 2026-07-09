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

# Exponer puerto
EXPOSE 8000

# Comando de inicio: Migrar, cargar semillas y levantar con Gunicorn
CMD ["sh", "-c", "python manage.py migrate && python manage.py seed_data && gunicorn ecoalerta_backend.wsgi:application --bind 0.0.0.0:8000"]
