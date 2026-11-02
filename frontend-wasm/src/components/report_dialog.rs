use leptos::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use crate::api::{alerts_url, analyze_evidence_url, upload_url};
use crate::geo::{find_nearby_active_alert, get_browser_gps_position, reverse_geocode};
use crate::models::{
    AuthUser, CreateAlertPayload, EcoAlert, EcoCategory, EcoDistrict, EcoSeverity,
    GeminiAnalysisResponse,
};
use crate::storage::{get_last_report_time, set_last_report_time};

#[component]
pub fn ReportDialog(
    alerts: RwSignal<Vec<EcoAlert>>,
    show_new_report_dialog: RwSignal<bool>,
    is_report_mode: RwSignal<bool>,
    report_center: RwSignal<(f64, f64)>,
    current_user: RwSignal<Option<AuthUser>>,
    show_auth_dialog: RwSignal<bool>,
) -> impl IntoView {
    let title = create_rw_signal("".to_string());
    let description = create_rw_signal("".to_string());
    let address = create_rw_signal("".to_string());
    let category = create_rw_signal(EcoCategory::Mineria);
    let severity = create_rw_signal(EcoSeverity::Critico);
    let district = create_rw_signal(EcoDistrict::Chaupimarca);

    let photos = create_rw_signal(Vec::<String>::new());
    let is_uploading_photo = create_rw_signal(false);
    let is_submitting = create_rw_signal(false);
    let error_message = create_rw_signal(None::<String>);
    let is_geocoding = create_rw_signal(false);
    let is_locating = create_rw_signal(false);

    // Gemini AI Analysis state
    let is_analyzing_ai = create_rw_signal(false);
    let ai_notice = create_rw_signal(None::<(String, String)>); // (message, type: "info" | "warning" | "success")
    let is_blocked_by_ai = create_rw_signal(false);
    let ai_block_reason = create_rw_signal(None::<String>);

    let analyze_photo_with_ai = {
        let current_user = current_user;
        let is_analyzing_ai = is_analyzing_ai;
        let ai_notice = ai_notice;
        let is_blocked_by_ai = is_blocked_by_ai;
        let ai_block_reason = ai_block_reason;
        let title = title;
        let description = description;
        let category = category;
        let severity = severity;

        move |img_url: String| {
            let user_opt = current_user.get_untracked();
            let token = match user_opt {
                Some(u) => u.token,
                None => return,
            };

            is_analyzing_ai.set(true);
            ai_notice.set(None);

            spawn_local(async move {
                let payload = serde_json::json!({
                    "image_url": img_url
                });

                let res = gloo_net::http::Request::post(&analyze_evidence_url())
                    .header("Authorization", &format!("Token {}", token))
                    .header("Content-Type", "application/json")
                    .json(&payload);

                if let Ok(req) = res {
                    if let Ok(resp) = req.send().await {
                        if resp.ok() {
                            if let Ok(data) = resp.json::<GeminiAnalysisResponse>().await {
                                // 1. Check minor litter filter (basura menor aislada)
                                if data.is_minor_litter == Some(true)
                                    || data.rejection_reason.as_deref() == Some("minor_litter")
                                    || data.waste_scale.as_deref() == Some("basura_menor")
                                {
                                    is_blocked_by_ai.set(true);
                                    let reason = "La evidencia corresponde a basura menor cotidiana o una bolsa aislada. El camión recolector municipal pasa habitualmente según su cronograma. EcoAlerta está reservado exclusivamente para focos de contaminación masivos y emergencias ambientales graves en Pasco.".to_string();
                                    ai_block_reason.set(Some(reason));
                                    ai_notice.set(None);
                                    title.set("".to_string());
                                    description.set("".to_string());
                                } else if data.is_environmental_issue == Some(false)
                                    || data.rejection_reason.as_deref() == Some("not_environmental")
                                    || data.category.as_deref() == Some("ninguna")
                                {
                                    // 2. Check non-environmental filter (selfie, vehículo normal, interiores, paisaje limpio, etc.)
                                    is_blocked_by_ai.set(true);
                                    let detail = data.technical_analysis.unwrap_or_else(|| {
                                        "La fotografía no corresponde a un incidente o problema de contaminación ambiental en Cerro de Pasco.".to_string()
                                    });
                                    let reason = detail.trim().to_string();
                                    ai_block_reason.set(Some(reason));
                                    ai_notice.set(None);
                                    title.set("".to_string());
                                    description.set("".to_string());
                                } else {
                                    // Evidencia ambiental válida y confirmada
                                    is_blocked_by_ai.set(false);
                                    ai_block_reason.set(None);
                                    ai_notice.set(Some((
                                        "✨ Evidencia verificada como problema ambiental real con Gemini AI: Título, descripción y categoría sugeridos automáticamente.".to_string(),
                                        "success".to_string(),
                                    )));

                                    if let Some(st) = data.suggested_title {
                                        if !st.trim().is_empty() {
                                            title.set(st);
                                        }
                                    }

                                    if let Some(sd) = data.suggested_description {
                                        if !sd.trim().is_empty() {
                                            description.set(sd);
                                        }
                                    }

                                    if let Some(cat_str) = data.category {
                                        let cat_lower = cat_str.to_lowercase();
                                        if cat_lower.contains("miner") || cat_lower.contains("relave") || cat_lower.contains("tajo") {
                                            category.set(EcoCategory::Mineria);
                                        } else if cat_lower.contains("basur") || cat_lower.contains("desmonte") || cat_lower.contains("residuo") || cat_lower.contains("plastic") {
                                            category.set(EcoCategory::Basura);
                                        } else if cat_lower.contains("agua") || cat_lower.contains("rio") || cat_lower.contains("lagun") || cat_lower.contains("efluente") {
                                            category.set(EcoCategory::Agua);
                                        } else if cat_lower.contains("aire") || cat_lower.contains("humo") || cat_lower.contains("emision") {
                                            category.set(EcoCategory::Aire);
                                        }
                                    }

                                    if let Some(sev_str) = data.severity {
                                        let sev_lower = sev_str.to_lowercase();
                                        if sev_lower.contains("critico") {
                                            severity.set(EcoSeverity::Critico);
                                        } else if sev_lower.contains("medio") {
                                            severity.set(EcoSeverity::Medio);
                                        } else if sev_lower.contains("bajo") {
                                            severity.set(EcoSeverity::Bajo);
                                        }
                                    }
                                }
                            }
                        } else {
                            let status = resp.status();
                            if status == 429 {
                                ai_notice.set(Some((
                                    "Límite de análisis de IA alcanzado. Por protección de la plataforma, por favor espera un momento antes de reintentar.".to_string(),
                                    "warning".to_string(),
                                )));
                            } else if let Ok(err_val) = resp.json::<serde_json::Value>().await {
                                let err_msg = err_val.get("error").and_then(|v| v.as_str())
                                    .or_else(|| err_val.get("detail").and_then(|v| v.as_str()));
                                if let Some(err_msg) = err_msg {
                                    if !err_msg.contains("desactivado") && !err_msg.contains("no está configurada") {
                                        ai_notice.set(Some((format!("Nota IA: {}", err_msg), "warning".to_string())));
                                    }
                                }
                            }
                        }
                    }
                }
                is_analyzing_ai.set(false);
            });
        }
    };

    // Synchronize modal state to JS to avoid Leaflet map events during report creation & reverse geocode once
    create_effect(move |_| {
        let is_open = show_new_report_dialog.get();
        let _ = js_sys::eval(&format!("window.isReportModalOpen = {};", is_open));
        if is_open {
            let (lat, lng) = report_center.get_untracked();
            if address.get_untracked().trim().is_empty() {
                is_geocoding.set(true);
                spawn_local(async move {
                    let (addr_str, dist) = reverse_geocode(lat, lng).await;
                    address.set(addr_str);
                    district.set(dist);
                    is_geocoding.set(false);
                });
            }
        }
    });

    // Cooldown seconds remaining
    let cooldown_remaining = move || {
        if let Some(last_ts) = get_last_report_time() {
            let now = js_sys::Date::now();
            let diff_sec = ((now - last_ts) / 1000.0) as i64;
            if diff_sec < 60 {
                return (60 - diff_sec) as u32;
            }
        }
        0
    };

    // Duplicate detection within 50 meters
    let nearby_duplicate = move || {
        let (lat, lng) = report_center.get();
        let cat = category.get();
        let list = alerts.get();
        find_nearby_active_alert(&list, lat, lng, cat, 50.0)
    };

    // Upload photo via JS helper (supports camera vs gallery input)
    let upload_from_input = {
        let photos = photos.clone();
        let error_message = error_message.clone();
        let is_uploading_photo = is_uploading_photo.clone();
        let analyze_photo_with_ai = analyze_photo_with_ai.clone();

        move |input_id: &'static str| {
            if photos.get().len() >= 2 {
                error_message.set(Some("Límite alcanzado: máximo 2 fotografías por reporte.".to_string()));
                return;
            }

            is_uploading_photo.set(true);
            error_message.set(None);

            let photos = photos.clone();
            let error_message = error_message.clone();
            let is_uploading_photo = is_uploading_photo.clone();
            let analyze_photo_with_ai = analyze_photo_with_ai.clone();

            spawn_local(async move {
                let script = format!("window.ecoUpload.uploadFile('{}', '{}')", input_id, upload_url());
                match js_sys::eval(&script) {
                    Ok(promise_val) => {
                        let promise = js_sys::Promise::from(promise_val);
                        match JsFuture::from(promise).await {
                            Ok(val) => {
                                if let Some(url_str) = val.as_string() {
                                    let is_first = photos.get_untracked().is_empty();
                                    photos.update(|p| p.push(url_str.clone()));
                                    if is_first {
                                        analyze_photo_with_ai(url_str);
                                    }
                                } else {
                                    error_message.set(Some("Respuesta inesperada al subir foto.".to_string()));
                                }
                            }
                            Err(e) => {
                                let msg = if let Some(err) = e.dyn_ref::<js_sys::Error>() {
                                    err.message().into()
                                } else if let Some(s) = e.as_string() {
                                    s
                                } else {
                                    "Error al procesar la fotografía.".to_string()
                                };
                                error_message.set(Some(msg));
                            }
                        }
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Error invocando cargador: {:?}", e)));
                    }
                }
                let _ = js_sys::eval(&format!(
                    "if(document.getElementById('{}')) document.getElementById('{}').value = ''",
                    input_id, input_id
                ));
                is_uploading_photo.set(false);
            });
        }
    };

    let upload_camera_photo = {
        let upload = upload_from_input.clone();
        move |_| upload("report-camera-input")
    };

    let upload_gallery_photo = {
        let upload = upload_from_input.clone();
        move |_| upload("report-gallery-input")
    };

    let remove_photo = move |idx: usize| {
        photos.update(|list| {
            if idx < list.len() {
                list.remove(idx);
            }
        });
        if photos.get().is_empty() {
            is_blocked_by_ai.set(false);
            ai_block_reason.set(None);
            ai_notice.set(None);
        }
    };

    let clear_all_photos = move |_: ()| {
        photos.set(Vec::new());
        is_blocked_by_ai.set(false);
        ai_block_reason.set(None);
        ai_notice.set(None);
        title.set("".to_string());
        description.set("".to_string());
    };

    let locate_current_gps = move |_| {
        is_locating.set(true);
        spawn_local(async move {
            match get_browser_gps_position().await {
                Ok((lat, lng)) => {
                    report_center.set((lat, lng));
                    let _ = js_sys::eval(&format!("window.ecoMap.setUserLocation({}, {});", lat, lng));
                    let (addr_str, dist) = reverse_geocode(lat, lng).await;
                    address.set(addr_str);
                    district.set(dist);
                }
                Err(err_msg) => {
                    error_message.set(Some(err_msg));
                }
            }
            is_locating.set(false);
        });
    };

    let submit_report = move |_| {
        let user = match current_user.get() {
            Some(u) => u,
            None => {
                show_new_report_dialog.set(false);
                show_auth_dialog.set(true);
                return;
            }
        };

        if is_blocked_by_ai.get() {
            error_message.set(Some(
                "No puedes enviar este reporte: la evidencia fotográfica fue rechazada por el filtro de IA al no constituir un problema ambiental.".to_string()
            ));
            return;
        }

        if is_analyzing_ai.get() {
            error_message.set(Some(
                "Por favor espera a que finalice el análisis de la fotografía con Gemini AI antes de enviar.".to_string()
            ));
            return;
        }

        if cooldown_remaining() > 0 {
            error_message.set(Some(format!(
                "Debes esperar {} segundos antes de enviar otro reporte ambiental.",
                cooldown_remaining()
            )));
            return;
        }

        let t = title.get();
        let d = description.get();
        let addr = address.get();

        if t.trim().is_empty() {
            error_message.set(Some("Por favor ingresa un título para el reporte.".to_string()));
            return;
        }

        if addr.trim().is_empty() {
            error_message.set(Some("Por favor especifica la dirección o referencia.".to_string()));
            return;
        }

        let (lat, lng) = report_center.get();
        let photo_list = photos.get();
        let img1 = photo_list.get(0).cloned();
        let img2 = photo_list.get(1).cloned();

        let payload = CreateAlertPayload {
            title: t.trim().to_string(),
            description: if d.trim().is_empty() {
                "Reporte ciudadano sin descripción extendida.".to_string()
            } else {
                d.trim().to_string()
            },
            category: category.get(),
            severity: severity.get(),
            district: district.get(),
            address: addr.trim().to_string(),
            latitude: lat,
            longitude: lng,
            image_url: img1,
            image_url_2: img2,
        };

        is_submitting.set(true);
        error_message.set(None);

        spawn_local(async move {
            let req_res = gloo_net::http::Request::post(&alerts_url())
                .header("Authorization", &format!("Token {}", user.token))
                .header("Content-Type", "application/json")
                .json(&payload);

            match req_res {
                Ok(req) => match req.send().await {
                    Ok(resp) => {
                        is_submitting.set(false);
                        if resp.ok() {
                            if let Ok(new_alert) = resp.json::<EcoAlert>().await {
                                alerts.update(|list| {
                                    if let Some(existing) = list.iter_mut().find(|a| a.id == new_alert.id) {
                                        *existing = new_alert;
                                    } else {
                                        list.insert(0, new_alert);
                                    }
                                });
                            }
                            set_last_report_time(js_sys::Date::now());
                            title.set("".to_string());
                            description.set("".to_string());
                            photos.set(Vec::new());
                            is_blocked_by_ai.set(false);
                            ai_block_reason.set(None);
                            ai_notice.set(None);
                            show_new_report_dialog.set(false);
                            is_report_mode.set(false);
                        } else {
                            if resp.status() == 429 {
                                error_message.set(Some("Has alcanzado el límite de reportes por hora (máximo 10).".to_string()));
                            } else if let Ok(err_val) = resp.json::<serde_json::Value>().await {
                                if let Some(det) = err_val.get("detail").and_then(|v| v.as_str()) {
                                    error_message.set(Some(det.to_string()));
                                } else if let Some(err_str) = err_val.get("error").and_then(|v| v.as_str()) {
                                    error_message.set(Some(err_str.to_string()));
                                } else {
                                    error_message.set(Some(format!("Error en el registro ({})", resp.status())));
                                }
                            } else {
                                error_message.set(Some("Error al registrar la alerta en el servidor.".to_string()));
                            }
                        }
                    }
                    Err(e) => {
                        is_submitting.set(false);
                        error_message.set(Some(format!("Error de conexión: {}", e)));
                    }
                },
                Err(e) => {
                    is_submitting.set(false);
                    error_message.set(Some(format!("Error serializando alerta: {}", e)));
                }
            }
        });
    };

    view! {
        {move || show_new_report_dialog.get().then(|| {
            view! {
                <div class="fixed inset-0 bg-slate-900/50 backdrop-blur-sm flex items-end sm:items-center justify-center p-0 sm:p-4 z-50 animate-in fade-in duration-150">
                    <div class="bg-white w-full sm:max-w-lg rounded-t-3xl sm:rounded-2xl border-t sm:border border-slate-200/90 p-5 sm:p-6 shadow-[0_-12px_48px_rgba(0,0,0,0.22)] max-h-[92vh] overflow-y-auto space-y-4">
                        // Top Drag handle for mobile
                        <div class="flex justify-center -mt-2 mb-1 sm:hidden">
                            <div class="w-12 h-1.5 bg-slate-300 rounded-full"></div>
                        </div>

                        // Accessible inputs: Camara directa y Galeria/Archivos
                        <input
                            type="file"
                            id="report-camera-input"
                            accept="image/*"
                            capture="environment"
                            class="sr-only"
                            on:change=upload_camera_photo
                        />
                        <input
                            type="file"
                            id="report-gallery-input"
                            accept="image/*"
                            class="sr-only"
                            on:change=upload_gallery_photo
                        />

                        // Modal Header
                        <div class="flex items-center justify-between border-b border-slate-100 pb-3">
                            <div>
                                <h2 class="text-base font-bold text-slate-900">
                                    "Registrar Nuevo Incidente Ambiental"
                                </h2>
                                <p class="text-[11px] text-slate-500 font-medium">"Cerro de Pasco · Fiscalización Ciudadana"</p>
                            </div>
                            <button
                                class="text-slate-400 hover:text-slate-700 p-1 rounded-lg hover:bg-slate-100 transition"
                                on:click=move |_| {
                                    show_new_report_dialog.set(false);
                                    is_report_mode.set(false);
                                }
                            >
                                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <line x1="18" y1="6" x2="6" y2="18"></line>
                                    <line x1="6" y1="6" x2="18" y2="18"></line>
                                </svg>
                            </button>
                        </div>

                        // Anti-Spam Cooldown Banner
                        {move || {
                            let cd = cooldown_remaining();
                            (cd > 0).then(|| view! {
                                <div class="p-3 bg-amber-50 rounded-xl border border-amber-200 flex items-start space-x-2.5">
                                    <svg class="w-4 h-4 text-amber-600 mt-0.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                        <circle cx="12" cy="12" r="10"></circle>
                                        <polyline points="12 6 12 12 16 14"></polyline>
                                    </svg>
                                    <p class="text-[11px] text-amber-800 leading-snug font-medium">
                                        {format!("Enfriamiento anti-spam activo: Espera {} segundos antes de enviar otro reporte ambiental.", cd)}
                                    </p>
                                </div>
                            })
                        }}

                        // Duplicate Warning Banner
                        {move || nearby_duplicate().map(|(dup_alert, meters)| view! {
                            <div class="p-3 bg-red-50 rounded-xl border border-red-200 flex items-start space-x-2.5">
                                <svg class="w-4 h-4 text-red-600 mt-0.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"></path>
                                    <line x1="12" y1="9" x2="12" y2="13"></line>
                                    <line x1="12" y1="17" x2="12.01" y2="17"></line>
                                </svg>
                                <div>
                                    <p class="text-[11px] font-bold text-red-800">
                                        {format!("Alerta similar ya registrada a {} metros", meters)}
                                    </p>
                                    <p class="text-[11px] text-red-700 leading-snug">
                                        {format!("Existe una alerta activa en esta zona (\"{title}\"). Evitemos duplicar incidentes.", title = dup_alert.title)}
                                    </p>
                                </div>
                            </div>
                        })}

                        // Error Banner
                        {move || error_message.get().map(|msg| view! {
                            <div class="p-3 bg-red-50 rounded-xl border border-red-200 text-[11px] text-red-700 font-medium">
                                {msg}
                            </div>
                        })}

                        // ==================== PASO 1: EVIDENCIA FOTOGRÁFICA (AL INICIO) ====================
                        <div class="p-4 bg-slate-50/90 rounded-2xl border border-slate-200/90 space-y-3 shadow-2xs">
                            // Card Header (Flat Apple Style - No Wrapping Blowouts)
                            <div class="flex items-center justify-between gap-2 border-b border-slate-200/60 pb-2.5">
                                <div class="flex items-center space-x-2 min-w-0">
                                    <span class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-slate-900 text-white uppercase tracking-wider shrink-0 whitespace-nowrap">
                                        "Paso 1"
                                    </span>
                                    <h3 class="text-xs sm:text-sm font-bold text-slate-900 truncate flex items-center space-x-1">
                                        <span>"Evidencia Fotográfica"</span>
                                        <span class="text-rose-500 font-bold shrink-0">*</span>
                                    </h3>
                                </div>
                                <div class="flex items-center space-x-2 shrink-0">
                                    {move || (!photos.get().is_empty()).then(|| {
                                        let reanalyze = analyze_photo_with_ai.clone();
                                        view! {
                                            <button
                                                type="button"
                                                class="text-purple-700 hover:text-purple-900 font-semibold text-[11px] flex items-center space-x-1 cursor-pointer transition"
                                                disabled=move || is_analyzing_ai.get()
                                                title="Volver a analizar con IA"
                                                on:click=move |_| {
                                                    if let Some(first_url) = photos.get_untracked().first() {
                                                        reanalyze(first_url.clone());
                                                    }
                                                }
                                            >
                                                <i class="fa-solid fa-sparkles text-[10px]"></i>
                                                <span class="hidden sm:inline">"Reanalizar con IA"</span>
                                                <span class="sm:hidden">"Reanalizar"</span>
                                            </button>
                                        }
                                    })}
                                    <span class="bg-white text-slate-600 px-2.5 py-0.5 rounded-full border border-slate-200 font-mono font-bold text-[10px] shadow-2xs">
                                        {move || format!("{}/2", photos.get().len())}
                                    </span>
                                </div>
                            </div>

                            <p class="text-[11px] text-slate-500 leading-relaxed">
                                "Sube la fotografía del incidente. Google Gemini Vision AI analizará la gravedad y autocompletará los detalles automáticamente."
                            </p>

                            // Photo Slots & Upload Grid - Pure Symmetrical Layout
                            <div class="grid grid-cols-2 gap-3 pt-0.5">
                                // Slot 1 & 2 Photos
                                {move || photos.get().into_iter().enumerate().map(|(idx, url)| {
                                    let is_first = idx == 0;
                                    view! {
                                        <div class=move || {
                                            if is_first && is_analyzing_ai.get() {
                                                "relative aspect-square w-full rounded-2xl overflow-hidden border-2 border-purple-500 bg-slate-950 shadow-md gemini-scanning-card"
                                            } else {
                                                "relative aspect-square w-full rounded-2xl overflow-hidden border border-slate-200 bg-slate-100 shadow-2xs"
                                            }
                                        }>
                                            <img src=url.clone() alt="Evidencia" class="w-full h-full object-cover" />

                                            // Scanning Overlay
                                            {move || (is_first && is_analyzing_ai.get()).then(|| view! {
                                                <div class="absolute inset-0 pointer-events-none">
                                                    <div class="gemini-laser-line"></div>
                                                    <div class="absolute inset-0 bg-gradient-to-b from-sky-400/20 via-transparent to-purple-500/25"></div>
                                                </div>
                                            })}

                                            // Delete Button
                                            <button
                                                type="button"
                                                class="absolute top-2 right-2 w-6 h-6 bg-slate-900/80 hover:bg-rose-600 active:scale-95 text-white rounded-full flex items-center justify-center backdrop-blur-xs transition shadow-sm z-30 cursor-pointer"
                                                title="Eliminar foto"
                                                on:click=move |_| remove_photo(idx)
                                            >
                                                <i class="fa-solid fa-xmark text-[11px]"></i>
                                            </button>
                                        </div>
                                    }
                                }).collect_view()}

                                // Upload Placeholder Slot (if less than 2 photos)
                                {move || if photos.get().len() == 0 {
                                    if is_uploading_photo.get() {
                                        view! {
                                            <div class="col-span-2 py-6 px-4 rounded-2xl border-2 border-dashed border-purple-300 bg-purple-50/50 flex flex-col items-center justify-center space-y-2">
                                                <span class="inline-block w-6 h-6 border-2 border-purple-600 border-t-transparent rounded-full animate-spin"></span>
                                                <span class="font-bold text-xs text-purple-800">"Optimizando y subiendo fotografía..."</span>
                                                <span class="text-[10px] text-purple-600 font-medium">"Reduciendo tamaño para transmisión rápida"</span>
                                            </div>
                                        }.into_view()
                                    } else {
                                        // 0 photos: Side-by-side Camera and Gallery buttons
                                        view! {
                                            <label
                                                for="report-camera-input"
                                                class=move || format!(
                                                    "col-span-1 py-4 px-3 rounded-2xl border-2 border-emerald-300 hover:border-emerald-500 bg-gradient-to-b from-emerald-50/80 to-white text-slate-800 flex flex-col items-center justify-center space-y-2 transition active:scale-[0.98] cursor-pointer shadow-xs group select-none text-center {}",
                                                    if is_uploading_photo.get() || is_analyzing_ai.get() { "pointer-events-none opacity-50" } else { "" }
                                                )
                                            >
                                                <div class="w-11 h-11 rounded-2xl bg-emerald-600 text-white flex items-center justify-center shadow-xs group-hover:scale-105 transition">
                                                    <i class="fa-solid fa-camera text-base"></i>
                                                </div>
                                                <div>
                                                    <span class="block font-bold text-xs text-slate-900 leading-tight">"Tomar Foto"</span>
                                                    <span class="block text-[10px] text-emerald-700 font-semibold mt-0.5">"Abrir cámara"</span>
                                                </div>
                                            </label>

                                            <label
                                                for="report-gallery-input"
                                                class=move || format!(
                                                    "col-span-1 py-4 px-3 rounded-2xl border-2 border-slate-200 hover:border-slate-400 bg-gradient-to-b from-slate-50/80 to-white text-slate-800 flex flex-col items-center justify-center space-y-2 transition active:scale-[0.98] cursor-pointer shadow-xs group select-none text-center {}",
                                                    if is_uploading_photo.get() || is_analyzing_ai.get() { "pointer-events-none opacity-50" } else { "" }
                                                )
                                            >
                                                <div class="w-11 h-11 rounded-2xl bg-slate-100 text-slate-700 flex items-center justify-center shadow-xs group-hover:scale-105 transition">
                                                    <i class="fa-solid fa-images text-base"></i>
                                                </div>
                                                <div>
                                                    <span class="block font-bold text-xs text-slate-900 leading-tight">"Subir Galería"</span>
                                                    <span class="block text-[10px] text-slate-500 font-medium mt-0.5">"Elegir archivo"</span>
                                                </div>
                                            </label>
                                        }.into_view()
                                    }
                                } else if photos.get().len() == 1 {
                                    // 1 photo: Symmetric second slot with camera & gallery choices
                                    view! {
                                        <div class="aspect-square w-full rounded-2xl border-2 border-dashed border-slate-300 bg-white p-2.5 flex flex-col justify-between items-center text-center shadow-2xs">
                                            {move || if is_uploading_photo.get() {
                                                view! {
                                                    <div class="h-full flex flex-col items-center justify-center space-y-1">
                                                        <span class="inline-block w-5 h-5 border-2 border-purple-600 border-t-transparent rounded-full animate-spin"></span>
                                                        <span class="font-bold text-purple-700 text-[11px] mt-1">"Subiendo..."</span>
                                                    </div>
                                                }.into_view()
                                            } else {
                                                view! {
                                                    <span class="text-[10px] font-bold text-slate-500 uppercase tracking-wider mt-0.5">
                                                        "+ 2da Evidencia"
                                                    </span>
                                                    <div class="w-full grid grid-cols-2 gap-1.5 mb-0.5">
                                                        <label
                                                            for="report-camera-input"
                                                            class="py-2 px-1 rounded-xl bg-emerald-50 hover:bg-emerald-100 border border-emerald-300 text-emerald-800 flex flex-col items-center justify-center transition active:scale-95 cursor-pointer shadow-2xs"
                                                            title="Tomar foto con cámara"
                                                        >
                                                            <i class="fa-solid fa-camera text-xs mb-0.5 text-emerald-600"></i>
                                                            <span class="text-[9px] font-bold">"Cámara"</span>
                                                        </label>
                                                        <label
                                                            for="report-gallery-input"
                                                            class="py-2 px-1 rounded-xl bg-slate-100 hover:bg-slate-200 border border-slate-300 text-slate-700 flex flex-col items-center justify-center transition active:scale-95 cursor-pointer shadow-2xs"
                                                            title="Elegir de galería"
                                                        >
                                                            <i class="fa-solid fa-images text-xs mb-0.5 text-slate-600"></i>
                                                            <span class="text-[9px] font-bold">"Galería"</span>
                                                        </label>
                                                    </div>
                                                }.into_view()
                                            }}
                                        </div>
                                    }.into_view()
                                } else {
                                    view! { <span class="hidden"></span> }.into_view()
                                }}
                            </div>

                            // AI Scanning Animated Card
                            {move || is_analyzing_ai.get().then(|| view! {
                                <div class="p-3.5 rounded-2xl bg-slate-900 text-white border border-slate-800 shadow-sm flex items-center space-x-3.5 animate-in fade-in duration-200">
                                    <div class="w-9 h-9 rounded-xl bg-purple-600 flex items-center justify-center text-white shrink-0 shadow-sm">
                                        <span class="text-base gemini-sparkle-icon">"✦"</span>
                                    </div>
                                    <div class="flex-1 min-w-0">
                                        <div class="flex items-center space-x-2">
                                            <span class="font-bold text-xs tracking-tight text-white">
                                                "Google Gemini Vision AI"
                                            </span>
                                            <span class="px-2 py-0.5 rounded-full text-[9px] font-bold bg-purple-900/60 text-purple-300 uppercase tracking-wider animate-pulse">
                                                "Escaneando"
                                            </span>
                                        </div>
                                        <p class="text-[11px] text-slate-300 mt-0.5 leading-snug">
                                            "Analizando píxeles y evaluando impacto ambiental..."
                                        </p>
                                        <div class="w-full bg-white/15 h-1 rounded-full mt-2 overflow-hidden">
                                            <div class="bg-purple-400 h-full w-full rounded-full animate-pulse"></div>
                                        </div>
                                    </div>
                                </div>
                            })}

                            // Redesigned Symmetrical Rejection Card (clean aesthetics, balanced layout)
                            {move || ai_block_reason.get().map(|reason| view! {
                                <div class="rounded-2xl bg-rose-50/90 border border-rose-200/90 p-3.5 shadow-2xs space-y-2.5 animate-in fade-in duration-150">
                                    <div class="flex items-center justify-between gap-2">
                                        <div class="flex items-center space-x-2 min-w-0">
                                            <div class="w-7 h-7 rounded-lg bg-rose-100 text-rose-600 flex items-center justify-center shrink-0">
                                                <i class="fa-solid fa-ban text-xs"></i>
                                            </div>
                                            <div class="min-w-0">
                                                <h4 class="text-xs font-bold text-rose-950 truncate leading-tight">
                                                    "Evidencia No Permitida"
                                                </h4>
                                                <p class="text-[10px] font-semibold text-rose-600 uppercase tracking-wide">
                                                    "Filtro Ambiental"
                                                </p>
                                            </div>
                                        </div>
                                        <button
                                            type="button"
                                            class="px-2.5 py-1 rounded-full text-[11px] font-semibold bg-white hover:bg-rose-100 active:bg-rose-200 text-rose-700 border border-rose-200 transition shrink-0 cursor-pointer shadow-2xs flex items-center space-x-1"
                                            on:click=move |_| clear_all_photos(())
                                        >
                                            <i class="fa-solid fa-trash-can text-[10px]"></i>
                                            <span>"Descartar"</span>
                                        </button>
                                    </div>
                                    
                                    <div class="p-3 bg-white rounded-xl border border-rose-100/90 text-xs text-rose-950 leading-relaxed font-normal">
                                        {reason}
                                    </div>

                                    <div class="flex items-center space-x-2 text-[11px] text-rose-700/90 pt-0.5">
                                        <span class="text-xs shrink-0">"💡"</span>
                                        <span class="leading-snug">"Sube fotos de acumulación de basura, relaves mineros o aguas residuales."</span>
                                    </div>
                                </div>
                            })}

                            // Success Notice Banner (when AI approved the evidence)
                            {move || ai_notice.get().map(|(msg, kind)| {
                                if kind == "success" {
                                    view! {
                                        <div class="rounded-2xl bg-emerald-50/90 border border-emerald-200 p-3.5 shadow-xs flex items-center space-x-3 text-emerald-950 animate-in fade-in">
                                            <div class="w-8 h-8 rounded-xl bg-emerald-100 border border-emerald-200 flex items-center justify-center text-emerald-700 shrink-0 font-bold text-sm">
                                                "✓"
                                            </div>
                                            <div class="flex-1 min-w-0">
                                                <span class="text-[10px] font-bold uppercase tracking-wider text-emerald-600 block">
                                                    "Verificado por Gemini AI"
                                                </span>
                                                <p class="text-xs text-emerald-900 font-medium leading-snug">
                                                    {msg}
                                                </p>
                                            </div>
                                        </div>
                                    }.into_view()
                                } else {
                                    view! {
                                        <div class="p-3 rounded-xl border border-amber-200 bg-amber-50 text-amber-900 text-xs leading-relaxed flex items-start space-x-2">
                                            <span class="shrink-0 text-sm">"⚠️"</span>
                                            <span class="font-medium flex-1">{msg}</span>
                                        </div>
                                    }.into_view()
                                }
                            })}
                        </div>

                        // ==================== PASO 2: UBICACIÓN Y CONFIRMACIÓN ====================
                        <div class="space-y-3.5 pt-1">
                            <div class="flex items-center space-x-2 pb-0.5">
                                <span class="px-2 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider bg-slate-900 text-white shrink-0 whitespace-nowrap">
                                    "Paso 2"
                                </span>
                                <h3 class="text-xs sm:text-sm font-bold text-slate-900 truncate">
                                    "Ubicación y Datos del Incidente"
                                </h3>
                            </div>

                            // Location & GPS row
                            <div class="p-3 bg-slate-50 rounded-xl border border-slate-200/80 space-y-2">
                                <div class="flex items-center justify-between text-[11px]">
                                    <span class="font-bold text-slate-400 uppercase tracking-wider text-[10px]">"Ubicación en el Mapa"</span>
                                    <span class="font-mono text-slate-700 font-medium">
                                        {move || {
                                            let (clat, clng) = report_center.get();
                                            format!("{:.4}, {:.4}", clat, clng)
                                        }}
                                    </span>
                                </div>
                                <div class="flex items-center space-x-2">
                                    <button
                                        type="button"
                                        class="py-1.5 px-3 bg-white hover:bg-slate-100 text-slate-700 border border-slate-200 rounded-lg text-xs font-semibold flex items-center space-x-1.5 transition shadow-xs"
                                        disabled=move || is_locating.get()
                                        on:click=locate_current_gps
                                    >
                                        {move || if is_locating.get() {
                                            view! {
                                                <span class="inline-block w-3 h-3 border-2 border-slate-400 border-t-slate-800 rounded-full animate-spin"></span>
                                                <span>"Localizando..."</span>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <svg class="w-3.5 h-3.5 text-sky-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                    <circle cx="12" cy="12" r="3"/>
                                                    <path d="M12 2v3m0 14v3M2 12h3m14 0h3"/>
                                                </svg>
                                                <span>"Usar mi ubicación GPS"</span>
                                            }.into_view()
                                        }}
                                    </button>
                                    {move || is_geocoding.get().then(|| view! {
                                        <span class="text-[11px] text-slate-400 italic">"Identificando calle..."</span>
                                    })}
                                </div>
                            </div>

                            // Form Inputs
                            <div class="space-y-3.5 text-xs">
                                <div>
                                    <label class="block text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-1.5">
                                        "Dirección o Referencia"
                                    </label>
                                    <input
                                        type="text"
                                        placeholder="Ej. Av. Los Próceres s/n, Yanacancha"
                                        class="w-full bg-slate-50 text-slate-800 rounded-xl px-3 py-2.5 border border-slate-200 focus:bg-white focus:border-slate-400 focus:outline-none focus:ring-2 focus:ring-slate-100 transition"
                                        prop:value=move || address.get()
                                        on:input=move |ev| address.set(event_target_value(&ev))
                                    />
                                </div>

                                <div>
                                    <label class="block text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-1.5">
                                        "Título del Incidente"
                                    </label>
                                    <input
                                        type="text"
                                        placeholder="Ej. Emisión de polvareda o vertido de efluentes"
                                        class="w-full bg-slate-50 text-slate-800 rounded-xl px-3 py-2.5 border border-slate-200 focus:bg-white focus:border-slate-400 focus:outline-none focus:ring-2 focus:ring-slate-100 transition"
                                        prop:value=move || title.get()
                                        on:input=move |ev| title.set(event_target_value(&ev))
                                    />
                                </div>

                                <div>
                                    <label class="block text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-1.5">
                                        "Categoría de Impacto"
                                    </label>
                                    <div class="grid grid-cols-2 gap-2">
                                        {EcoCategory::all().iter().map(|&cat| {
                                            let is_sel = move || category.get() == cat;
                                            view! {
                                                <button
                                                    type="button"
                                                    class=move || {
                                                        if is_sel() {
                                                            "py-2 px-3 rounded-xl border border-slate-900 bg-slate-900 text-white font-semibold text-xs flex items-center space-x-2 transition"
                                                        } else {
                                                            "py-2 px-3 rounded-xl border border-slate-200 bg-slate-50 text-slate-600 text-xs flex items-center space-x-2 hover:bg-slate-100 transition"
                                                        }
                                                    }
                                                    on:click=move |_| category.set(cat)
                                                >
                                                    <span class="text-sm">{cat.icon()}</span>
                                                    <span>{cat.display_name()}</span>
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>

                                <div>
                                    <label class="block text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-1.5">
                                        "Nivel de Severidad"
                                    </label>
                                    <div class="grid grid-cols-3 gap-2">
                                        {EcoSeverity::all().iter().map(|&sev| {
                                            let is_sel = move || severity.get() == sev;
                                            view! {
                                                <button
                                                    type="button"
                                                    class=move || {
                                                        if is_sel() {
                                                            match sev {
                                                                EcoSeverity::Critico => "py-2 px-2 rounded-xl border border-red-300 bg-red-50 text-red-700 font-semibold text-xs text-center transition",
                                                                EcoSeverity::Medio => "py-2 px-2 rounded-xl border border-amber-300 bg-amber-50 text-amber-700 font-semibold text-xs text-center transition",
                                                                EcoSeverity::Bajo => "py-2 px-2 rounded-xl border border-emerald-300 bg-emerald-50 text-emerald-700 font-semibold text-xs text-center transition",
                                                            }
                                                        } else {
                                                            "py-2 px-2 rounded-xl border border-slate-200 bg-slate-50 text-slate-500 text-xs text-center hover:bg-slate-100 transition"
                                                        }
                                                    }
                                                    on:click=move |_| severity.set(sev)
                                                >
                                                    {match sev {
                                                        EcoSeverity::Critico => "Crítico 💀",
                                                        EcoSeverity::Medio => "Medio ⚠️",
                                                        EcoSeverity::Bajo => "Bajo 🍃",
                                                    }}
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>

                                <div>
                                    <label class="block text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-1.5">
                                        "Descripción Detallada"
                                    </label>
                                    <textarea
                                        rows="3"
                                        placeholder="Describe la afectación en la zona..."
                                        class="w-full bg-slate-50 text-slate-800 rounded-xl px-3 py-2.5 border border-slate-200 focus:bg-white focus:border-slate-400 focus:outline-none focus:ring-2 focus:ring-slate-100 resize-none text-xs transition"
                                        prop:value=move || description.get()
                                        on:input=move |ev| description.set(event_target_value(&ev))
                                    ></textarea>
                                </div>
                            </div>
                        </div>

                        // Footer Buttons
                        <div class="flex justify-end space-x-2.5 border-t border-slate-100 pt-3">
                            <button
                                type="button"
                                class="px-3.5 py-2 text-xs font-semibold text-slate-500 hover:text-slate-800 transition"
                                on:click=move |_| {
                                    show_new_report_dialog.set(false);
                                    is_report_mode.set(false);
                                }
                            >
                                "Cancelar"
                            </button>
                            <button
                                type="button"
                                class=move || {
                                    if is_blocked_by_ai.get() {
                                        "px-4 py-2 text-xs font-semibold bg-red-600/90 hover:bg-red-700 cursor-not-allowed text-white rounded-xl shadow-xs transition flex items-center space-x-1.5"
                                    } else {
                                        "px-4 py-2 text-xs font-semibold bg-slate-900 hover:bg-slate-800 disabled:bg-slate-400 text-white rounded-xl shadow-xs transition flex items-center space-x-1.5"
                                    }
                                }
                                disabled=move || {
                                    let cd = cooldown_remaining();
                                    let is_dup = nearby_duplicate().is_some();
                                    is_submitting.get() || (cd > 0) || is_dup || is_blocked_by_ai.get() || is_analyzing_ai.get()
                                }
                                on:click=submit_report
                            >
                                {move || if is_submitting.get() {
                                    view! {
                                        <span class="inline-block w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
                                        <span>"Registrando..."</span>
                                    }
                                } else if is_analyzing_ai.get() {
                                    view! {
                                        <span class="inline-block w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
                                        <span>"Analizando evidencia..."</span>
                                    }
                                } else if is_blocked_by_ai.get() {
                                    view! {
                                        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                            <circle cx="12" cy="12" r="10"></circle>
                                            <line x1="4.93" y1="4.93" x2="19.07" y2="19.07"></line>
                                        </svg>
                                        <span>"Bloqueado: No es Ambiental"</span>
                                    }
                                } else if cooldown_remaining() > 0 {
                                    let cd = cooldown_remaining();
                                    view! {
                                        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                            <circle cx="12" cy="12" r="10"></circle>
                                            <polyline points="12 6 12 12 16 14"></polyline>
                                        </svg>
                                        <span>{format!("Espera ({}s)", cd)}</span>
                                    }
                                } else if nearby_duplicate().is_some() {
                                    view! {
                                        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                            <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"></path>
                                        </svg>
                                        <span>"Alerta Duplicada"</span>
                                    }
                                } else {
                                    view! {
                                        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                            <path d="M22 2 11 13"/>
                                            <polygon points="22 2 15 22 11 13 2 9 22 2"/>
                                        </svg>
                                        <span>"Publicar Reporte"</span>
                                    }
                                }}
                            </button>
                        </div>
                    </div>
                </div>
            }
        })}
    }
}
