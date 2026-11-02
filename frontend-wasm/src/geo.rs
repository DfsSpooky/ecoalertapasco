use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use crate::models::{EcoAlert, EcoCategory, EcoDistrict};

pub fn haversine_distance_meters(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371000.0; // Earth radius in meters
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r * c
}

pub struct Landmark {
    pub name: &'static str,
    pub lat: f64,
    pub lng: f64,
    pub district: EcoDistrict,
}

pub const LANDMARKS: &[Landmark] = &[
    // Yanacancha
    Landmark { name: "Avenida Los Próceres (Frente al Hospital Regional), Yanacancha", lat: -10.6480, lng: -76.2490, district: EcoDistrict::Yanacancha },
    Landmark { name: "Avenida Los Próceres (Cerca al Terminal Terrestre), Yanacancha", lat: -10.6550, lng: -76.2410, district: EcoDistrict::Yanacancha },
    Landmark { name: "Avenida Daniel Alcides Carrión (Bulevar de Yanacancha), Yanacancha", lat: -10.6520, lng: -76.2420, district: EcoDistrict::Yanacancha },
    Landmark { name: "Avenida Los Incas 480, Yanacancha", lat: -10.6565, lng: -76.2380, district: EcoDistrict::Yanacancha },
    Landmark { name: "Carretera Central (Sector Puente Río San Juan), Yanacancha", lat: -10.6420, lng: -76.2360, district: EcoDistrict::Yanacancha },
    Landmark { name: "Avenida El Minero (Sector Industrial La Esperanza), Yanacancha", lat: -10.6590, lng: -76.2480, district: EcoDistrict::Yanacancha },
    Landmark { name: "Jirón Progreso (Cerca a San Juan), Yanacancha", lat: -10.6440, lng: -76.2390, district: EcoDistrict::Yanacancha },
    Landmark { name: "Avenida Los Incas (Esquina Los Próceres), Yanacancha", lat: -10.6500, lng: -76.2450, district: EcoDistrict::Yanacancha },
    Landmark { name: "Avenida Las Américas (Esquina Av. 6 de Diciembre), Yanacancha", lat: -10.6640, lng: -76.2550, district: EcoDistrict::Yanacancha },
    Landmark { name: "Calle Mariátegui (Cerca a Av. Las Américas), Yanacancha", lat: -10.6630, lng: -76.2540, district: EcoDistrict::Yanacancha },
    Landmark { name: "Jirón Moreno (Cerca a Av. Las Américas), Yanacancha", lat: -10.6650, lng: -76.2560, district: EcoDistrict::Yanacancha },
    Landmark { name: "Jirón Diputación, Yanacancha", lat: -10.6700, lng: -76.2540, district: EcoDistrict::Yanacancha },
    Landmark { name: "Jirón Ramón Castilla, Yanacancha", lat: -10.6710, lng: -76.2530, district: EcoDistrict::Yanacancha },
    Landmark { name: "Avenida Daniel Alcides Carrión (Cerca a la UNDAC), Yanacancha", lat: -10.6690, lng: -76.2520, district: EcoDistrict::Yanacancha },
    Landmark { name: "Avenida El Minero (Entrada a la UNDAC), Yanacancha", lat: -10.6715, lng: -76.2510, district: EcoDistrict::Yanacancha },
    Landmark { name: "Avenida Los Incas (Corte Superior de Justicia), Yanacancha", lat: -10.6670, lng: -76.2460, district: EcoDistrict::Yanacancha },

    // Chaupimarca
    Landmark { name: "Jirón Daniel Alcides Carrión 112 (Plaza Daniel Alcides Carrión), Chaupimarca", lat: -10.6781, lng: -76.2545, district: EcoDistrict::Chaupimarca },
    Landmark { name: "Avenida Bolognesi 302 (Frente a Municipalidad), Chaupimarca", lat: -10.6790, lng: -76.2550, district: EcoDistrict::Chaupimarca },
    Landmark { name: "Jirón Alfonso Ugarte (Ribera Este de Laguna Patarcocha), Chaupimarca", lat: -10.6655, lng: -76.2525, district: EcoDistrict::Chaupimarca },
    Landmark { name: "Jirón San Martín (Ribera Oeste de Laguna Patarcocha), Chaupimarca", lat: -10.6680, lng: -76.2530, district: EcoDistrict::Chaupimarca },
    Landmark { name: "Jirón Hilario Cabrera 215 (Cerca al Mercado Central), Chaupimarca", lat: -10.6795, lng: -76.2590, district: EcoDistrict::Chaupimarca },
    Landmark { name: "Jirón Lampa s/n (Borde del Tajo Abierto), Chaupimarca", lat: -10.6720, lng: -76.2570, district: EcoDistrict::Chaupimarca },
    Landmark { name: "Jirón Circunvalación Túpac Amaru (Borde del Tajo Abierto), Chaupimarca", lat: -10.6730, lng: -76.2620, district: EcoDistrict::Chaupimarca },
    Landmark { name: "Jirón Tarapacá 310, Chaupimarca", lat: -10.6750, lng: -76.2555, district: EcoDistrict::Chaupimarca },
    Landmark { name: "Asentamiento Humano Uliachin Sector Alto, Chaupimarca", lat: -10.6860, lng: -76.2620, district: EcoDistrict::Chaupimarca },
    Landmark { name: "Jirón Junín (Cerca a San Cristóbal), Chaupimarca", lat: -10.6770, lng: -76.2570, district: EcoDistrict::Chaupimarca },

    // Simón Bolívar
    Landmark { name: "Presa de Relaves Quiulacocha, Simón Bolívar", lat: -10.7022, lng: -76.2871, district: EcoDistrict::SimonBolivar },
    Landmark { name: "Presa de Relaves Quiulacocha Sector Sur, Simón Bolívar", lat: -10.7040, lng: -76.2900, district: EcoDistrict::SimonBolivar },
    Landmark { name: "Sector Ocroyoc, Presa de Relaves, Simón Bolívar", lat: -10.6935, lng: -76.2990, district: EcoDistrict::SimonBolivar },
    Landmark { name: "Comunidad de Champamarca Sector Agropecuario, Simón Bolívar", lat: -10.7100, lng: -76.2730, district: EcoDistrict::SimonBolivar },
    Landmark { name: "Planta de Lixiviados Sector San Juan, Simón Bolívar", lat: -10.6920, lng: -76.2750, district: EcoDistrict::SimonBolivar },
    Landmark { name: "Sector Bellavista s/n, Simón Bolívar", lat: -10.6970, lng: -76.2750, district: EcoDistrict::SimonBolivar },
    Landmark { name: "Carretera Central KM 4.5 (Entrada a Quiulacocha), Simón Bolívar", lat: -10.6880, lng: -76.2720, district: EcoDistrict::SimonBolivar },
];

pub fn deduce_closest_landmark(lat: f64, lng: f64) -> (&'static str, EcoDistrict) {
    let mut min_d = f64::INFINITY;
    let mut closest = (LANDMARKS[0].name, LANDMARKS[0].district);
    for lm in LANDMARKS {
        let d = (lm.lat - lat).powi(2) + (lm.lng - lng).powi(2);
        if d < min_d {
            min_d = d;
            closest = (lm.name, lm.district);
        }
    }
    closest
}

pub async fn reverse_geocode(lat: f64, lng: f64) -> (String, EcoDistrict) {
    let url = format!(
        "https://nominatim.openstreetmap.org/reverse?format=json&lat={}&lon={}&zoom=18&addressdetails=1",
        lat, lng
    );

    let req_res = gloo_net::http::Request::get(&url)
        .send()
        .await;

    if let Ok(resp) = req_res {
        if resp.ok() {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(addr) = json.get("address").and_then(|a| a.as_object()) {
                    let road = addr.get("road")
                        .or_else(|| addr.get("suburb"))
                        .or_else(|| addr.get("pedestrian"))
                        .or_else(|| addr.get("neighbourhood"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("Calle Sin Nombre");

                    let full_str = serde_json::to_string(addr).unwrap_or_default().to_lowercase();
                    let district = if full_str.contains("yanacancha") {
                        EcoDistrict::Yanacancha
                    } else if full_str.contains("simon bolivar") || full_str.contains("simón bolívar") {
                        EcoDistrict::SimonBolivar
                    } else if full_str.contains("chaupimarca") {
                        EcoDistrict::Chaupimarca
                    } else {
                        deduce_closest_landmark(lat, lng).1
                    };

                    let addr_str = format!("{}, {}", road, district.display_name());
                    return (addr_str, district);
                }
            }
        }
    }

    let (name, district) = deduce_closest_landmark(lat, lng);
    (name.to_string(), district)
}

pub fn find_nearby_active_alert(
    alerts: &[EcoAlert],
    lat: f64,
    lng: f64,
    category: EcoCategory,
    max_meters: f64,
) -> Option<(EcoAlert, u32)> {
    for a in alerts {
        if a.status == "solved" || a.status == "dismissed" {
            continue;
        }
        if a.category != category {
            continue;
        }
        let d = haversine_distance_meters(lat, lng, a.latitude, a.longitude);
        if d <= max_meters {
            return Some((a.clone(), d.round() as u32));
        }
    }
    None
}

pub async fn get_browser_gps_position() -> Result<(f64, f64), String> {
    let promise_val = js_sys::eval("window.ecoGeo.getCurrentPosition()")
        .map_err(|e| format!("{:?}", e))?;

    let promise = js_sys::Promise::from(promise_val);
    let js_val = JsFuture::from(promise)
        .await
        .map_err(|e| {
            if let Some(s) = e.as_string() {
                s
            } else if let Some(err) = e.dyn_ref::<js_sys::Error>() {
                err.message().into()
            } else {
                "Error al obtener ubicación GPS".to_string()
            }
        })?;

    let lat = js_sys::Reflect::get(&js_val, &JsValue::from_str("lat"))
        .map_err(|_| "Coordenada latitud no encontrada")?
        .as_f64()
        .ok_or("Latitud inválida")?;

    let lng = js_sys::Reflect::get(&js_val, &JsValue::from_str("lng"))
        .map_err(|_| "Coordenada longitud no encontrada")?
        .as_f64()
        .ok_or("Longitud inválida")?;

    Ok((lat, lng))
}
