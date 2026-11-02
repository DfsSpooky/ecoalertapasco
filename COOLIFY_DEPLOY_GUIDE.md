# Guía de Despliegue de EcoAlerta en Coolify 🌱

Esta guía detalla el paso a paso para desplegar la aplicación completa en producción (**Base de Datos PostgreSQL**, **Backend Django**, y **Frontend WebAssembly Rust/Leptos**) en tu instancia de **Coolify**.

---

## 📋 Arquitectura de Despliegue Recomendada

Para producción, se recomienda configurar **tres recursos individuales** dentro del mismo proyecto y entorno en Coolify. Esto permite aislar responsabilidades, escalar independientemente el backend/frontend, y dejar que Coolify gestione automáticamente los certificados SSL (HTTPS).

```mermaid
graph TD
    User([Ciudadano / Autoridad]) -->|HTTPS| FE(Frontend: Rust + Leptos WASM / Nginx)
    FE <-->|WebSocket Tiempo Real WSS /ws/map/| BE(Backend: Django Channels + Uvicorn ASGI)
    BE -->|Conexión Interna| DB[(Base de Datos: PostgreSQL)]
    BE -.->|Opcional Multiréplica| RD[(Redis Channel Layer)]
```

---

## 🛠️ Paso 1: Desplegar la Base de Datos (PostgreSQL)

Coolify provee bases de datos integradas "one-click" que se configuran muy fácilmente.

1. En el panel de Coolify, ve al proyecto/entorno donde deseas realizar el despliegue.
2. Haz clic en **+ New** y selecciona **Database** -> **PostgreSQL**.
3. Configura los parámetros:
   - **Name**: `ecoalerta-db`
   - **Postgres Database**: `ecoalerta`
   - **Postgres User**: `ecoalerta_user`
   - **Postgres Password**: *(Ingresa una contraseña segura)*
4. Guarda e inicia la base de datos.
5. **Copia los siguientes datos de conexión** del panel de la base de datos:
   - **Host Interno** (generalmente una dirección interna de Docker, ej. `postgresql-12345:5432`) o el nombre del servicio.
   - **Puerto Interno** (`5432`)
   - **Usuario**, **Contraseña** y **Nombre de la Base de Datos**.

---

## 🐍 Paso 2: Desplegar el Backend (Django)

El backend de Django se compilará y ejecutará usando el [Dockerfile](file:///Users/miguel/Documents/Proyectos/EcoAlerta/backend/Dockerfile) que se encuentra en la carpeta `/backend`.

1. En Coolify, haz clic en **+ New** -> **Application** -> **Public Repository** (o Private Repository si es privado).
2. Pega la URL de tu repositorio Git y la rama correspondiente.
3. Configura los ajustes de construcción:
   - **Build Pack**: Selecciona `Dockerfile`.
   - **Docker Build Path (Build Source)**: Cámbialo a `/backend` (esto es crucial para que use el Dockerfile del backend).
   - **Ports Excluded/Exposed**: Indica el puerto `8000`.
4. Asigna un dominio para la API en el campo **Domains** (ej. `https://api.ecoalerta.tudominio.com`). Coolify gestionará el certificado SSL automáticamente.
5. Ve a la pestaña **Environment Variables** (Variables de Entorno) y añade lo siguiente:
   - `DEBUG`: `False`
   - `SECRET_KEY`: *(Genera una cadena aleatoria larga y segura para producción)*
   - `ALLOWED_HOSTS`: `api.ecoalerta.tudominio.com,localhost,127.0.0.1`
   - `CORS_ALLOWED_ORIGINS`: `https://ecoalerta.tudominio.com`
   - `CSRF_TRUSTED_ORIGINS`: `https://ecoalerta.tudominio.com,https://api.ecoalerta.tudominio.com`
   - `DB_HOST`: *(El host interno de la base de datos de Coolify)*
   - `DB_NAME`: `ecoalerta`
   - `DB_USER`: `ecoalerta_user`
   - `DB_PASSWORD`: *(La contraseña que definiste en el Paso 1)*
   - `DB_PORT`: `5432`
   - `GOOGLE_CLIENT_ID`: *(Opcional — Tu Web Client ID de Google Cloud Console para habilitar "Continuar con Google")*
   - `TURNSTILE_SITEKEY`: *(Opcional — Por defecto usa la clave de prueba universal `1x00000000000000000000AA` para desarrollo/staging; en producción ingresa tu Site Key de Cloudflare)*
   - `TURNSTILE_SECRET_KEY`: *(Opcional — Por defecto usa la clave de prueba universal `1x0000000000000000000000000000000AA`; en producción ingresa tu Secret Key de Cloudflare)*
   - `TURNSTILE_ENABLED`: `true` *(o `false` si deseas desactivar temporalmente el anti-bot)*
   - `REDIS_URL`: *(Opcional — Ej. `redis://redis-interno:6379/0` si despliegas múltiples réplicas; con 1 contenedor Uvicorn funciona nativamente sin requerir Redis externo)*
6. En **Persistent Storages** (Almacenamiento Persistente), monta un volumen para conservar las fotografías de evidencia WebP entre redespliegues:
   - **Mount Path**: `/app/media`
7. Haz clic en **Deploy**. El contenedor ejecutará de forma segura las migraciones (`migrate --noinput`), compilará estáticos (`collectstatic`), registrará el catálogo inicial de estaciones de acopio únicamente si la tabla está vacía (`seed_waste_points --if-empty`, sin borrar jamás denuncias reales) e iniciará el servidor ASGI **Uvicorn** con soporte nativo para HTTP + WebSockets (`wss://`).

---

## ⚡ Paso 3: Desplegar el Frontend WebAssembly (`frontend-wasm`)

El frontend moderno en Rust + Leptos WebAssembly se compila usando `frontend-wasm/Dockerfile` y se sirve mediante **Nginx** con compresión Gzip para `.wasm` e inyección dinámica de `BACKEND_URL`.

1. En Coolify, haz clic en **+ New** -> **Application** -> Selecciona el mismo repositorio Git.
2. Configura los ajustes de construcción:
   - **Build Pack**: Selecciona `Dockerfile`.
   - **Base Directory (Build Context)**: Cámbialo a `/frontend-wasm`.
   - **Dockerfile Location**: `/Dockerfile` (relativo a `/frontend-wasm`).
   - **Ports Exposes**: Indica el puerto `80`.
3. Asigna tu dominio principal en el campo **Domains** (ej. `https://ecoalerta.tudominio.com`).
4. Ve a **Environment Variables** (funciona tanto en Runtime como en Build Time gracias al entrypoint de Nginx):
   - Añade `BACKEND_URL=https://api.ecoalerta.tudominio.com` (sin barra final `/`).
5. Haz clic en **Deploy**. Coolify compilará el binario WebAssembly en modo `--release` e inyectará automáticamente `BACKEND_URL` en `<meta name="ecoalerta-api-url">` al iniciar Nginx.

---

## 🔍 Verificación del Despliegue

Una vez completado el despliegue de los 3 recursos, valida lo siguiente:
1. Accede a `https://ecoalerta.tudominio.com`. Deberías visualizar el mapa interactivo de Cerro de Pasco y el Dashboard con Glassmorphism.
2. Realiza un reporte de prueba en el mapa para confirmar que los datos se guarden correctamente en la base de datos PostgreSQL a través de la API del backend.
3. Intenta iniciar sesión como autoridad para validar que las peticiones HTTP y la autenticación por token funcionen correctamente bajo HTTPS sin errores de CORS.

---

## 💡 Consejos de Resolución de Problemas (Troubleshooting)

### ❌ Error: "Mixed Content" (Petición bloqueada por el navegador)
* **Causa**: El frontend corre bajo `https://` pero estás intentando llamar al backend usando `http://`.
* **Solución**: Asegúrate de que configuraste `BACKEND_URL` con `https://` en los **Build Arguments** del frontend en Coolify, y que volviste a desplegar la aplicación para aplicar el cambio.

### ❌ Error: "CORS (Cross-Origin Resource Sharing)"
* **Causa**: Las peticiones del dominio del frontend son rechazadas por el backend de Django.
* **Solución**: El backend en [settings.py](file:///Users/miguel/Documents/Proyectos/EcoAlerta/backend/ecoalerta_backend/settings.py) ya tiene configurado `CORS_ALLOW_ALL_ORIGINS = True`. Si prefieres restringirlo por seguridad en producción, puedes instalar `django-cors-headers` y listar explícitamente tu dominio en `CORS_ALLOWED_ORIGINS` dentro de `settings.py`.

### ❌ Las migraciones no se ejecutan o fallan al iniciar el Backend
* **Causa**: El backend intentó arrancar antes de que la base de datos PostgreSQL estuviera lista para recibir conexiones.
* **Solución**: Puedes reiniciar el servicio del backend desde el panel de Coolify para forzar un nuevo intento de conexión y migración.

---

## 💻 Configuración para Pruebas Locales (Multipass / VPS Local)

Si estás simulando el VPS en tu Mac usando **Multipass** (con la IP `192.168.252.3`), sigue estos pasos para probar sin un dominio real:

1. **Editar el archivo `/etc/hosts` en tu Mac**:
   Abre la terminal de tu Mac y edita el archivo de hosts del sistema:
   ```bash
   sudo nano /etc/hosts
   ```
   Añade la siguiente línea al final para redireccionar los dominios locales a tu VM:
   ```text
   192.168.252.3 ecoalerta.local api.ecoalerta.local
   ```
   Guarda el archivo (`Ctrl + O`, `Enter` y luego `Ctrl + X`).

2. **Configurar los dominios en Coolify**:
   * **Backend**: En el campo **Domains**, pon `http://api.ecoalerta.local` (usa `http://` en lugar de `https://`).
   * **Frontend**: En el campo **Domains**, pon `http://ecoalerta.local` (usa `http://` en lugar de `https://`).

3. **Inyectar la URL del Backend en el Frontend**:
   * En la pestaña **Build Arguments** del Frontend en Coolify, añade la variable:
     `BACKEND_URL=http://api.ecoalerta.local`

4. **Probar la aplicación**:
   * Abre tu navegador en la Mac y accede a `http://ecoalerta.local` para ver el frontend.
   * La aplicación se comunicará de forma transparente con el backend en `http://api.ecoalerta.local`.
