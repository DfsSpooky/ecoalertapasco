#!/bin/sh
set -e

# Si se define BACKEND_URL en las variables de entorno del contenedor (ej. Coolify),
# inyectarla dinámicamente en <meta name="ecoalerta-api-url" content="..."> antes de iniciar Nginx.
if [ -n "${BACKEND_URL:-}" ] && [ -f /usr/share/nginx/html/index.html ]; then
    CLEAN_URL=$(printf '%s' "$BACKEND_URL" | sed 's:/*$::')
    sed -i "s|<meta name=\"ecoalerta-api-url\" content=\"[^\"]*\"|<meta name=\"ecoalerta-api-url\" content=\"${CLEAN_URL}\"|g" /usr/share/nginx/html/index.html
fi

exec nginx -g "daemon off;"
