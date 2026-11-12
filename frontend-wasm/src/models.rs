use serde::de::Deserializer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EcoCategory {
    Mineria,
    Basura,
    Agua,
    Aire,
}

impl EcoCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            EcoCategory::Mineria => "Minería",
            EcoCategory::Basura => "Basura",
            EcoCategory::Agua => "Agua",
            EcoCategory::Aire => "Aire",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            EcoCategory::Mineria => "⛏️",
            EcoCategory::Basura => "🗑️",
            EcoCategory::Agua => "💧",
            EcoCategory::Aire => "💨",
        }
    }

    #[allow(dead_code)]
    pub fn color_hex(&self) -> &'static str {
        match self {
            EcoCategory::Mineria => "#F59E0B", // Amber
            EcoCategory::Basura => "#10B981",  // Emerald
            EcoCategory::Agua => "#0288D1",    // Blue
            EcoCategory::Aire => "#8B5CF6",    // Purple
        }
    }

    pub fn color_class(&self) -> &'static str {
        match self {
            EcoCategory::Mineria => "bg-amber-50 text-amber-800 border-amber-200",
            EcoCategory::Basura => "bg-emerald-50 text-emerald-800 border-emerald-200",
            EcoCategory::Agua => "bg-sky-50 text-sky-800 border-sky-200",
            EcoCategory::Aire => "bg-purple-50 text-purple-800 border-purple-200",
        }
    }

    pub fn all() -> &'static [EcoCategory] {
        &[
            EcoCategory::Mineria,
            EcoCategory::Basura,
            EcoCategory::Agua,
            EcoCategory::Aire,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EcoSeverity {
    Critico,
    Medio,
    Bajo,
}

impl EcoSeverity {
    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            EcoSeverity::Critico => "Crítico 💀",
            EcoSeverity::Medio => "Medio ⚠️",
            EcoSeverity::Bajo => "Bajo 🍃",
        }
    }

    #[allow(dead_code)]
    pub fn color_hex(&self) -> &'static str {
        match self {
            EcoSeverity::Critico => "#EF4444", // Red 500
            EcoSeverity::Medio => "#F59E0B",   // Amber 500
            EcoSeverity::Bajo => "#10B981",    // Emerald 500
        }
    }

    pub fn all() -> &'static [EcoSeverity] {
        &[EcoSeverity::Critico, EcoSeverity::Medio, EcoSeverity::Bajo]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EcoDistrict {
    Chaupimarca,
    Yanacancha,
    SimonBolivar,
}

impl EcoDistrict {
    pub fn display_name(&self) -> &'static str {
        match self {
            EcoDistrict::Chaupimarca => "Chaupimarca",
            EcoDistrict::Yanacancha => "Yanacancha",
            EcoDistrict::SimonBolivar => "Simón Bolívar",
        }
    }

    pub fn all() -> &'static [EcoDistrict] {
        &[
            EcoDistrict::Chaupimarca,
            EcoDistrict::Yanacancha,
            EcoDistrict::SimonBolivar,
        ]
    }
}

fn deserialize_id_from_any<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum IdHelper {
        Int(i64),
        Str(String),
    }
    match IdHelper::deserialize(deserializer)? {
        IdHelper::Int(i) => Ok(i.to_string()),
        IdHelper::Str(s) => Ok(s),
    }
}

fn default_status() -> String {
    "pending".to_string()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EcoAlert {
    #[serde(deserialize_with = "deserialize_id_from_any")]
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: EcoCategory,
    pub severity: EcoSeverity,
    pub district: EcoDistrict,
    pub address: String,
    pub latitude: f64,
    pub longitude: f64,
    pub created_at: String,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub image_url_2: Option<String>,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub confirmations_count: u32,
    #[serde(default)]
    pub has_user_confirmed: bool,
    #[serde(default)]
    pub reporter_name: Option<String>,
    #[serde(default)]
    pub resolution_comment: Option<String>,
    #[serde(default)]
    pub resolution_image_url: Option<String>,
    #[serde(default)]
    pub resolved_at: Option<String>,
    #[serde(default)]
    pub history_log: Option<String>,
}

impl EcoAlert {
    #[allow(dead_code)]
    pub fn parsed_history(&self) -> Vec<(String, String)> {
        match &self.history_log {
            Some(log) if !log.trim().is_empty() => {
                log.lines()
                    .filter(|line| line.contains('|'))
                    .map(|line| {
                        let mut parts = line.splitn(2, '|');
                        let date = parts.next().unwrap_or("").trim().to_string();
                        let action = parts.next().unwrap_or("").trim().to_string();
                        (date, action)
                    })
                    .collect()
            }
            _ => Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthUser {
    pub token: String,
    pub username: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub is_staff: bool,
    /// Conductor de camión recolector (grupo `recolectores` en el backend)
    #[serde(default)]
    pub is_collector: bool,
    #[serde(default)]
    pub photo_url: Option<String>,
    #[serde(default)]
    pub full_name: Option<String>,
}

impl AuthUser {
    pub fn is_authority(&self) -> bool {
        self.is_staff
    }

    pub fn role_label(&self) -> &'static str {
        if self.is_staff {
            "Autoridad / Fiscalizador"
        } else {
            "Ciudadano Vigilante"
        }
    }

    pub fn display_name(&self) -> String {
        if let Some(ref name) = self.full_name {
            if !name.trim().is_empty() {
                return name.trim().to_string();
            }
        }
        self.username.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleAuthPayload<'a> {
    pub credential: &'a str,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct GoogleAuthConfig {
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub is_configured: bool,
    #[serde(default)]
    pub app_name: String,
    #[serde(default)]
    pub turnstile_sitekey: Option<String>,
    #[serde(default)]
    pub turnstile_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginPayload<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterPayload<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub password: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turnstile_token: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAlertPayload {
    pub title: String,
    pub description: String,
    pub category: EcoCategory,
    pub severity: EcoSeverity,
    pub district: EcoDistrict,
    pub address: String,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url_2: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWastePointPayload {
    pub name: String,
    pub description: String,
    pub address: String,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(rename = "type")]
    pub waste_type: WasteType,
    pub fill_level: FillLevel,
    pub truck_schedule: String,
    pub image_url: String,
    pub recommendations: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfirmResponse {
    #[allow(dead_code)]
    pub id: serde_json::Value,
    pub confirmed: bool,
    pub confirmations_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStatus {
    Connecting,
    RealtimeWs,
    FallbackHttp,
    Offline,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsServerMessage {
    InitialSnapshot {
        alerts: Vec<EcoAlert>,
        waste_points: Vec<WastePoint>,
        #[serde(default)]
        routes: Vec<CollectionRoute>,
        #[serde(default)]
        truck_runs: Vec<TruckRun>,
        #[serde(default)]
        #[allow(dead_code)]
        server_time: Option<String>,
    },
    AlertUpserted {
        alert: EcoAlert,
    },
    AlertDeleted {
        id: String,
    },
    WastePointUpserted {
        waste_point: WastePoint,
    },
    WastePointDeleted {
        id: String,
    },
    RouteUpserted {
        route: CollectionRoute,
    },
    RouteDeleted {
        id: String,
    },
    TruckRunUpserted {
        run: TruckRun,
    },
    TruckRunEnded {
        id: String,
    },
    TruckPosition {
        position: TruckPositionUpdate,
    },
    AuthInvalid,
    Pong {
        #[serde(default)]
        #[allow(dead_code)]
        server_time: Option<String>,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SyncResponse {
    pub etag: String,
    pub alerts: Vec<EcoAlert>,
    pub waste_points: Vec<WastePoint>,
    #[serde(default)]
    pub routes: Vec<CollectionRoute>,
    #[serde(default)]
    pub truck_runs: Vec<TruckRun>,
    #[serde(default)]
    #[allow(dead_code)]
    pub server_time: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WasteType {
    General,
    Organic,
    Recyclable,
}

#[allow(dead_code)]
impl WasteType {
    pub fn all() -> &'static [WasteType] {
        &[WasteType::Recyclable, WasteType::Organic, WasteType::General]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            WasteType::General => "general",
            WasteType::Organic => "organic",
            WasteType::Recyclable => "recyclable",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            WasteType::General => "General",
            WasteType::Organic => "Orgánico",
            WasteType::Recyclable => "Reciclable",
        }
    }

    pub fn full_label(&self) -> &'static str {
        match self {
            WasteType::General => "Residuos Generales 🗑️ (Desechos Domésticos)",
            WasteType::Organic => "Orgánicos 🍂 (Cáscaras, Restos Vegetales, Compost)",
            WasteType::Recyclable => "Reciclables ♻️ (Plástico PET, Cartón, Latas, Pilas)",
        }
    }

    pub fn color_class(&self) -> &'static str {
        match self {
            WasteType::General => "bg-slate-100 text-slate-800 border-slate-200",
            WasteType::Organic => "bg-amber-50 text-amber-800 border-amber-200",
            WasteType::Recyclable => "bg-emerald-50 text-emerald-800 border-emerald-200",
        }
    }

    pub fn hex_color(&self) -> &'static str {
        match self {
            WasteType::General => "#475569",
            WasteType::Organic => "#D97706",
            WasteType::Recyclable => "#16A34A",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FillLevel {
    Low,
    Medium,
    Full,
}

impl Default for FillLevel {
    fn default() -> Self {
        FillLevel::Low
    }
}

impl FillLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            FillLevel::Low => "low",
            FillLevel::Medium => "medium",
            FillLevel::Full => "full",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            FillLevel::Low => "Bajo (< 30%)",
            FillLevel::Medium => "Medio (50% - 70%)",
            FillLevel::Full => "Lleno (Capacidad Máxima)",
        }
    }

    pub fn badge_class(&self) -> &'static str {
        match self {
            FillLevel::Low => "bg-emerald-50 text-emerald-700 border-emerald-200",
            FillLevel::Medium => "bg-amber-50 text-amber-700 border-amber-200",
            FillLevel::Full => "bg-red-50 text-red-700 border-red-200 animate-pulse",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WastePoint {
    pub id: serde_json::Value,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub address: String,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(rename = "type")]
    pub waste_type: WasteType,
    #[serde(default)]
    pub fill_level: FillLevel,
    #[serde(rename = "truck_schedule", default)]
    pub truck_schedule: String,
    #[serde(rename = "image_url", default)]
    pub image_url: String,
    #[serde(default)]
    pub recommendations: String,
}

impl WastePoint {
    pub fn id_string(&self) -> String {
        if let Some(s) = self.id.as_str() {
            s.to_string()
        } else if let Some(n) = self.id.as_i64() {
            n.to_string()
        } else {
            self.id.to_string()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RouteStop {
    pub id: serde_json::Value,
    #[serde(default)]
    pub order: u32,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default)]
    pub waste_point: Option<i64>,
    #[serde(default)]
    pub eta_offset_minutes: u32,
}

/// Ruta de recolección planificada por la autoridad (publicada desde `/api/routes/`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CollectionRoute {
    pub id: serde_json::Value,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub district: String,
    #[serde(default)]
    pub district_display: String,
    #[serde(default)]
    pub waste_type: String,
    #[serde(default = "default_route_color")]
    pub color: String,
    #[serde(default)]
    pub schedule: String,
    #[serde(default)]
    pub path: Vec<(f64, f64)>,
    #[serde(default)]
    pub default_truck_code: Option<String>,
    #[serde(default)]
    pub stops: Vec<RouteStop>,
    #[serde(default)]
    pub waypoints: Vec<(f64, f64)>,
    #[serde(default)]
    pub snap_to_roads: bool,
    #[serde(default)]
    pub default_truck: Option<i64>,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

fn default_true() -> bool {
    true
}

fn default_route_color() -> String {
    "#16a34a".to_string()
}

impl CollectionRoute {
    pub fn id_string(&self) -> String {
        if let Some(s) = self.id.as_str() {
            s.to_string()
        } else if let Some(n) = self.id.as_i64() {
            n.to_string()
        } else {
            self.id.to_string()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiAnalysisResponse {
    #[serde(default)]
    pub is_environmental_issue: Option<bool>,
    #[serde(default)]
    pub is_minor_litter: Option<bool>,
    #[serde(default)]
    pub rejection_reason: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub severity: Option<String>,
    #[serde(default)]
    pub waste_scale: Option<String>,
    #[serde(default)]
    pub suggested_title: Option<String>,
    #[serde(default)]
    pub suggested_description: Option<String>,
    #[serde(default)]
    pub technical_analysis: Option<String>,
    #[serde(default)]
    pub municipal_action: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub is_gemini_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiModelInfo {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub badge: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSettingsResponse {
    #[serde(default)]
    pub has_gemini_key: bool,
    #[serde(default)]
    pub masked_key: String,
    #[serde(default)]
    pub selected_model: String,
    #[serde(default)]
    pub available_models: Vec<GeminiModelInfo>,
    #[serde(default)]
    pub gemini_enabled: bool,
    #[serde(default)]
    pub google_client_id: Option<String>,
    #[serde(default)]
    pub google_auth_configured: Option<bool>,
    #[serde(default)]
    pub turnstile_sitekey: Option<String>,
    #[serde(default)]
    pub turnstile_secret_masked: Option<String>,
    #[serde(default)]
    pub has_turnstile_secret: Option<bool>,
    #[serde(default)]
    pub turnstile_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestGeminiResponse {
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub latency_ms: Option<u64>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub quota_tier: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMedal {
    pub id: String,
    pub title: String,
    pub desc: String,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub username: String,
    pub email: String,
    #[serde(default)]
    pub is_staff: bool,
    #[serde(default)]
    pub eco_points: u32,
    #[serde(default)]
    pub rank: String,
    #[serde(default)]
    pub level: u32,
    #[serde(default)]
    pub badge_icon: String,
    #[serde(default)]
    pub next_level_points: u32,
    #[serde(default)]
    pub total_reported: u32,
    #[serde(default)]
    pub total_solved: u32,
    #[serde(default)]
    pub total_in_inspection: u32,
    #[serde(default)]
    pub total_confirmations_given: u32,
    #[serde(default)]
    pub medals: Vec<UserMedal>,
}


/// Turno abierto de un camión recolector, tal como lo ve la ciudadanía (sin conductor ni placa).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TruckRun {
    pub id: i64,
    pub route: i64,
    #[serde(default)]
    pub route_name: String,
    #[serde(default)]
    pub route_color: String,
    pub truck_code: String,
    #[serde(default)]
    pub waste_type: String,
    pub status: String,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub last_latitude: Option<f64>,
    #[serde(default)]
    pub last_longitude: Option<f64>,
    #[serde(default)]
    pub last_heading: Option<f64>,
    #[serde(default)]
    pub last_speed_kmh: Option<f64>,
    #[serde(default)]
    pub last_position_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct TruckPositionUpdate {
    pub run_id: i64,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default)]
    pub heading: Option<f64>,
    #[serde(default)]
    pub speed_kmh: Option<f64>,
    #[serde(default)]
    pub at: Option<String>,
}

/// Turno visto por el propio conductor (`/api/runs/`).
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct DriverRun {
    pub id: i64,
    pub route: i64,
    #[serde(default)]
    pub route_name: String,
    pub truck: i64,
    #[serde(default)]
    pub truck_code: String,
    pub status: String,
    #[serde(default)]
    pub started_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct DriverTruck {
    pub id: i64,
    pub code: String,
    #[serde(default)]
    pub plate: String,
    #[serde(default)]
    pub waste_type: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct DriverContext {
    #[serde(default)]
    pub trucks: Vec<DriverTruck>,
    #[serde(default)]
    pub routes: Vec<CollectionRoute>,
    #[serde(default)]
    pub open_run: Option<DriverRun>,
}

/// Estado del rastreador GPS en JS (`window.ecoTracker.state`).
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct TrackerStatus {
    #[serde(default)]
    pub tracking: bool,
    #[serde(default)]
    pub gps: String,
    #[serde(default)]
    pub accuracy: Option<f64>,
    #[serde(default)]
    pub last_fix_at: Option<f64>,
    #[serde(default)]
    pub last_sent_at: Option<f64>,
    #[serde(default)]
    pub pending: u32,
    #[serde(default)]
    pub wake_lock: bool,
    #[serde(default)]
    pub server_status: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}


/// Estado del editor de rutas en JS (`window.ecoRouteEditor`).
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct RouteEditorState {
    #[serde(default)]
    pub waypoints: Vec<(f64, f64)>,
    #[serde(default)]
    pub path: Vec<(f64, f64)>,
    #[serde(default)]
    pub stops: Vec<EditorStop>,
    #[serde(default)]
    pub snap: bool,
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub distance_m: Option<f64>,
    #[serde(default)]
    pub duration_s: Option<f64>,
    #[serde(default)]
    pub snapping: bool,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub can_undo: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EditorStop {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default)]
    pub waste_point: Option<i64>,
    #[serde(default)]
    pub eta_offset_minutes: u32,
}

/// Camión con datos completos (vista de autoridad en `/api/trucks/`).
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct FleetTruck {
    pub id: i64,
    pub code: String,
    #[serde(default)]
    pub plate: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub waste_type: String,
    #[serde(default)]
    pub driver_username: Option<String>,
    #[serde(default)]
    pub is_active: bool,
}
