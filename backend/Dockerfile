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

# Sonda de Salud del Contenedor Backend
HEALTHCHECK --interval=15s --timeout=5s --start-period=15s --retries=3 \
    CMD python -c "import urllib.request; urllib.request.urlopen('http://127.0.0.1:8000/api/health/')" || exit 1

# Comando de inicio: Esperar base de datos, colectar estáticos, migrar, cargar semillas y levantar con Gunicorn
CMD ["sh", "-c", "python -c \"import time, os, psycopg2; host=os.environ.get('DB_HOST'); \nif host:\n  for i in range(30):\n    try:\n      conn=psycopg2.connect(host=host, port=os.environ.get('DB_PORT','5432'), dbname=os.environ.get('DB_NAME','ecoalerta'), user=os.environ.get('DB_USER','ecoalerta_user'), password=os.environ.get('DB_PASSWORD',''))\n      conn.close()\n      print('Conexion con PostgreSQL exitosa.')\n      break\n    except Exception as e:\n      print(f'Esperando PostgreSQL en {host}... ({e})')\n      time.sleep(2)\n\" && python manage.py collectstatic --noinput && python manage.py migrate && python manage.py seed_data && gunicorn ecoalerta_backend.wsgi:application --bind 0.0.0.0:8000"]

