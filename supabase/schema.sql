-- 1. Habilitar la extensión espacial PostGIS para manejo de geolocalización
CREATE EXTENSION IF NOT EXISTS postgis;

-- 2. Crear la tabla de alertas ecológicas
CREATE TABLE IF NOT EXISTS public.eco_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    category TEXT NOT NULL CHECK (category IN ('mineria', 'basura', 'agua', 'aire')),
    severity TEXT NOT NULL CHECK (severity IN ('critico', 'medio', 'bajo')),
    location GEOMETRY(Point, 4326) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT timezone('utc'::text, now()) NOT NULL,
    image_url TEXT
);

-- 3. Crear índice espacial GIST para búsquedas geográficas rápidas
CREATE INDEX IF NOT EXISTS eco_alerts_location_idx ON public.eco_alerts USING GIST (location);

-- 4. Habilitar Row Level Security (RLS)
ALTER TABLE public.eco_alerts ENABLE ROW LEVEL SECURITY;

-- 5. Crear políticas de seguridad para acceso abierto (ideal para plataformas de ciencia ciudadana)
CREATE POLICY "Permitir lectura pública a cualquier usuario" 
ON public.eco_alerts FOR SELECT 
USING (true);

CREATE POLICY "Permitir inserción pública de nuevos reportes" 
ON public.eco_alerts FOR INSERT 
WITH CHECK (true);

-- 6. Insertar datos de prueba (Seed Data) correspondientes a Cerro de Pasco
INSERT INTO public.eco_alerts (title, description, category, severity, location, created_at)
VALUES
    (
        'Relaves Mineros de Quiulacocha', 
        'Filtración y arrastre de sedimentos ácidos con metales pesados desde la desmontera hacia bofedales locales.', 
        'mineria', 
        'critico', 
        ST_SetSRID(ST_MakePoint(-76.2871, -10.7022), 4326),
        NOW() - INTERVAL '2 days'
    ),
    (
        'Polvo de Tajo Abierto Raul Rojas', 
        'Presencia de partículas de polvo en suspensión que provienen del movimiento de tierras del tajo abierto central.', 
        'mineria', 
        'critico', 
        ST_SetSRID(ST_MakePoint(-76.2570, -10.6720), 4326),
        NOW() - INTERVAL '5 days'
    ),
    (
        'Plomo en Suelo de Recreo Escolar', 
        'Medición de concentración de plomo excede los límites permisibles en áreas verdes contiguas a la escuela de Chaupimarca.', 
        'mineria', 
        'critico', 
        ST_SetSRID(ST_MakePoint(-76.2545, -10.6781), 4326),
        NOW() - INTERVAL '12 days'
    ),
    (
        'Aguas Ácidas en Laguna Patarcocha', 
        'Coloración verdosa inusual y emanación de gases sulfhídricos debido al vertido de aguas residuales y drenajes ácidos urbanos.', 
        'agua', 
        'critico', 
        ST_SetSRID(ST_MakePoint(-76.2525, -10.6655), 4326),
        NOW() - INTERVAL '3 days'
    ),
    (
        'Basural Acumulado en Av. El Minero', 
        'Punto crítico de acumulación de residuos sólidos domiciliarios y comerciales que bloquean la vereda peatonal.', 
        'basura', 
        'medio', 
        ST_SetSRID(ST_MakePoint(-76.2530, -10.6515), 4326),
        NOW() - INTERVAL '4 days'
    ),
    (
        'Monóxido y Ruido en Av. Italia', 
        'Emisiones vehiculares excesivas de buses y camiones de carga pesada en hora punta debido a embotellamiento crónico.', 
        'aire', 
        'medio', 
        ST_SetSRID(ST_MakePoint(-76.2595, -10.6690), 4326),
        NOW() - INTERVAL '7 days'
    );

-- Nota: Para mapear el punto geométrico a Latitud y Longitud en Flutter, puedes hacer una consulta regular
-- o crear una función SQL / vista que exponga latitud y longitud explícitamente:
--
-- CREATE OR REPLACE VIEW public.v_eco_alerts AS
-- SELECT 
--     id, 
--     title, 
--     description, 
--     category, 
--     severity, 
--     ST_Y(location) as latitude, 
--     ST_X(location) as longitude, 
--     created_at, 
--     image_url
-- FROM public.eco_alerts;
