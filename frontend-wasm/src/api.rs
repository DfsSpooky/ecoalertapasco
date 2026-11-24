use web_sys::window;

fn normalize_base_url(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.is_empty() || trimmed == "__BACKEND_URL__" {
        return None;
    }
    if trimmed == "same-origin" || trimmed == "/" {
        return window().and_then(|w| w.location().origin().ok());
    }
    Some(trimmed.to_string())
}

fn is_private_lan_ipv4(host: &str) -> bool {
    host.starts_with("192.168.")
        || host.starts_with("10.")
        || (host.starts_with("172.") && {
            host.split('.')
                .nth(1)
                .and_then(|o| o.parse::<u8>().ok())
                .map(|second| (16..=31).contains(&second))
                .unwrap_or(false)
        })
}

pub fn api_base_url() -> String {
    // 1. Configuración inyectada en runtime mediante <meta name="ecoalerta-api-url" content="...">
    if let Some(win) = window() {
        if let Some(doc) = win.document() {
            if let Ok(Some(meta)) = doc.query_selector("meta[name='ecoalerta-api-url']") {
                if let Some(content) = meta.get_attribute("content") {
                    if let Some(resolved) = normalize_base_url(&content) {
                        return resolved;
                    }
                }
            }
        }
    }

    // 2. Variable de entorno en tiempo de compilación (ej. BACKEND_URL en Coolify / Docker build)
    if let Some(compile_env) = option_env!("BACKEND_URL") {
        if let Some(resolved) = normalize_base_url(compile_env) {
            return resolved;
        }
    }

    // 3. Resolución inteligente según origen real del navegador (Cero Mixed-Content en HTTPS)
    if let Some(win) = window() {
        let loc = win.location();
        let protocol = loc.protocol().unwrap_or_else(|_| "http:".to_string());
        let hostname = loc.hostname().unwrap_or_default();

        if hostname == "localhost" || hostname == "127.0.0.1" {
            return "http://localhost:8000".to_string();
        }

        // Pruebas locales en celular dentro de red LAN Wi-Fi por HTTP
        if protocol == "http:" && is_private_lan_ipv4(&hostname) {
            return format!("http://{}:8000", hostname);
        }

        // Producción (HTTPS / Dominio público / Reverse Proxy): usar mismo origen por defecto
        if let Ok(origin) = loc.origin() {
            if !origin.is_empty() && origin != "null" {
                return origin.trim_end_matches('/').to_string();
            }
        }
    }

    "http://localhost:8000".to_string()
}

/// Expone la URL base resuelta en `window.__ECOALERTA_API_BASE__` para que utilidades JS
/// como el generador de Actas PDF y códigos QR apunten siempre al servidor real de producción.
pub fn sync_js_api_base() {
    let base = api_base_url();
    if let Ok(escaped) = serde_json::to_string(&base) {
        let _ = js_sys::eval(&format!("window.__ECOALERTA_API_BASE__ = {};", escaped));
    }
}

pub fn sync_url(etag: Option<&str>) -> String {
    match etag.map(str::trim).filter(|s| !s.is_empty()) {
        Some(tag) => format!("{}/api/sync/?etag={}", api_base_url(), tag),
        None => format!("{}/api/sync/", api_base_url()),
    }
}

/// Construye la URL WebSocket (`ws://` en HTTP local, `wss://` en HTTPS de producción)
/// a partir de `api_base_url()`, adjuntando el token DRF si el ciudadano/autoridad inició sesión.
pub fn ws_map_url(token: Option<&str>) -> String {
    let http_base = api_base_url();
    let ws_base = if let Some(rest) = http_base.strip_prefix("https://") {
        format!("wss://{}", rest)
    } else if let Some(rest) = http_base.strip_prefix("http://") {
        format!("ws://{}", rest)
    } else {
        format!("ws://{}", http_base)
    };

    match token.map(str::trim).filter(|t| !t.is_empty()) {
        Some(tok) => format!("{}/ws/map/?token={}", ws_base, js_sys::encode_uri_component(tok)),
        None => format!("{}/ws/map/", ws_base),
    }
}

pub fn alerts_url() -> String {
    format!("{}/api/alerts/", api_base_url())
}

pub fn login_url() -> String {
    format!("{}/api/auth/login/", api_base_url())
}

pub fn logout_url() -> String {
    format!("{}/api/auth/logout/", api_base_url())
}

pub fn register_url() -> String {
    format!("{}/api/auth/register/", api_base_url())
}

pub fn google_auth_url() -> String {
    format!("{}/api/auth/google/", api_base_url())
}

pub fn google_auth_config_url() -> String {
    format!("{}/api/auth/google/config/", api_base_url())
}

pub fn confirm_alert_url(alert_id: &str) -> String {
    format!("{}/api/alerts/{}/confirm/", api_base_url(), alert_id)
}

pub fn upload_url() -> String {
    format!("{}/api/upload/", api_base_url())
}

pub fn upload_report_url(is_private: bool) -> String {
    if is_private {
        format!("{}/api/upload/?scope=ciudadana&purpose=alert", api_base_url())
    } else {
        format!("{}/api/upload/?purpose=alert", api_base_url())
    }
}

#[allow(dead_code)]
pub fn upload_private_url() -> String {
    format!("{}/api/upload/?scope=ciudadana", api_base_url())
}

pub fn alert_evidence_url(alert_id: &str, slot: u8) -> String {
    format!("{}/api/alerts/{}/evidence/{}/", api_base_url(), alert_id, slot)
}

pub fn alerts_ciudadana_history_url() -> String {
    format!("{}/api/alerts/?domain=ciudadana&include_expired=1", api_base_url())
}

pub fn waste_points_url() -> String {
    format!("{}/api/waste-points/", api_base_url())
}

pub fn export_csv_url() -> String {
    format!("{}/api/alerts/export_csv/", api_base_url())
}

pub fn alert_detail_url(id: &str) -> String {
    format!("{}/api/alerts/{}/", api_base_url(), id)
}

pub fn waste_point_detail_url(id: &str) -> String {
    format!("{}/api/waste-points/{}/", api_base_url(), id)
}

pub fn analyze_evidence_url() -> String {
    format!("{}/api/ai/analyze-evidence/", api_base_url())
}

pub fn system_settings_url() -> String {
    format!("{}/api/system/settings/", api_base_url())
}

pub fn test_gemini_url() -> String {
    format!("{}/api/system/test-gemini/", api_base_url())
}

pub fn profile_url() -> String {
    format!("{}/api/auth/profile/", api_base_url())
}

#[allow(dead_code)]
pub fn verify_alert_url(id: &str) -> String {
    format!("{}/api/alerts/{}/verify/", api_base_url(), id)
}

#[allow(dead_code)]
pub fn health_url() -> String {
    format!("{}/api/health/", api_base_url())
}

pub fn driver_context_url() -> String {
    format!("{}/api/runs/driver-context/", api_base_url())
}

pub fn run_start_url() -> String {
    format!("{}/api/runs/start/", api_base_url())
}

/// `action` ∈ {"pause", "resume", "finish"}
pub fn run_action_url(run_id: i64, action: &str) -> String {
    format!("{}/api/runs/{}/{}/", api_base_url(), run_id, action)
}

pub fn routes_url() -> String {
    format!("{}/api/routes/", api_base_url())
}

pub fn route_detail_url(id: &str) -> String {
    format!("{}/api/routes/{}/", api_base_url(), id)
}

pub fn trucks_url() -> String {
    format!("{}/api/trucks/", api_base_url())
}

pub fn admin_url(path: &str) -> String {
    format!("{}/admin/{}", api_base_url(), path.trim_start_matches('/'))
}

pub fn is_protected_evidence_url(url: &str) -> bool {
    url.contains("/api/evidence/private/") || (url.contains("/api/alerts/") && url.contains("/evidence/"))
}

pub async fn fetch_protected_image_blob_url(url: &str, token: &str) -> Result<String, String> {
    let resp = gloo_net::http::Request::get(url)
        .header("Authorization", &format!("Token {}", token))
        .send()
        .await
        .map_err(|e| format!("Error de red: {:?}", e))?;

    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let bytes = resp
        .binary()
        .await
        .map_err(|e| format!("Error leyendo bytes: {:?}", e))?;

    let uint8 = js_sys::Uint8Array::from(bytes.as_slice());
    let array = js_sys::Array::new();
    array.push(&uint8.buffer());

    let props = web_sys::BlobPropertyBag::new();
    props.set_type("image/webp");

    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&array, &props)
        .map_err(|e| format!("Error creando Blob: {:?}", e))?;
    web_sys::Url::create_object_url_with_blob(&blob)
        .map_err(|e| format!("Error creando ObjectURL: {:?}", e))
}
