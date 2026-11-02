use leptos::*;
use crate::geo::haversine_distance_meters;
use crate::models::{AuthUser, FillLevel, WastePoint, WasteType};

fn format_distance(meters: f64) -> String {
    if meters < 1000.0 {
        format!("{:.0} m", meters)
    } else {
        format!("{:.1} km", meters / 1000.0)
    }
}

#[component]
pub fn WasteDetail(
    waste_points: RwSignal<Vec<WastePoint>>,
    selected_waste_point_id: RwSignal<Option<String>>,
    current_gps_or_center: RwSignal<(f64, f64)>,
    #[prop(optional)] current_user: Option<RwSignal<Option<AuthUser>>>,
    #[prop(optional)] show_auth_dialog: Option<RwSignal<bool>>,
) -> impl IntoView {
    let is_updating_fill = create_rw_signal(false);
    let fill_report_msg = create_rw_signal(None::<(bool, String)>);

    let report_fill_level = {
        let waste_points = waste_points.clone();
        move |pid: String, new_lvl: FillLevel| {
            if let Some(user_sig) = current_user {
                if user_sig.get().is_none() {
                    if let Some(dialog) = show_auth_dialog {
                        dialog.set(true);
                        return;
                    }
                }
            }

            is_updating_fill.set(true);
            fill_report_msg.set(None);

            let token_opt = current_user.and_then(|u| u.get().map(|user| user.token.clone()));
            let previous_lvl = waste_points
                .get_untracked()
                .iter()
                .find(|p| p.id_string() == pid)
                .map(|p| p.fill_level);

            // Optimistic UI update (reverted on server error)
            waste_points.update(|list| {
                if let Some(item) = list.iter_mut().find(|p| p.id_string() == pid) {
                    item.fill_level = new_lvl;
                }
            });

            let lvl_label = new_lvl.display_name();

            let pid_for_req = pid.clone();
            spawn_local(async move {
                let p_url = crate::api::waste_point_detail_url(&pid_for_req);
                let payload = serde_json::json!({
                    "fill_level": new_lvl,
                });

                let mut req = gloo_net::http::Request::patch(&p_url);
                if let Some(token) = token_opt {
                    req = req.header("Authorization", &format!("Token {}", token));
                }

                let mut ok = false;
                let mut err_detail = "No se pudo actualizar el contenedor en el servidor.".to_string();

                match req.json(&payload) {
                    Ok(r) => match r.send().await {
                        Ok(resp) => {
                            if resp.ok() {
                                ok = true;
                            } else if resp.status() == 401 || resp.status() == 403 {
                                err_detail = "Debes iniciar sesión para reportar el estado del contenedor.".to_string();
                            } else {
                                err_detail = format!("Error del servidor ({}) al actualizar el nivel.", resp.status());
                            }
                        }
                        Err(_) => {
                            err_detail = "Sin conexión con el servidor municipal. Intenta nuevamente.".to_string();
                        }
                    },
                    Err(_) => {}
                }

                if ok {
                    fill_report_msg.set(Some((true, format!("¡Gracias! Nivel actualizado a {} correctamente.", lvl_label))));
                } else {
                    if let Some(prev) = previous_lvl {
                        waste_points.update(|list| {
                            if let Some(item) = list.iter_mut().find(|p| p.id_string() == pid_for_req) {
                                item.fill_level = prev;
                            }
                        });
                    }
                    fill_report_msg.set(Some((false, err_detail)));
                }
                is_updating_fill.set(false);
            });
        }
    };

    view! {
        {move || {
            let selected_id_opt = selected_waste_point_id.get();
            let all = waste_points.get();
            let point_opt = selected_id_opt.and_then(|sid| all.into_iter().find(|p| p.id_string() == sid));

            point_opt.map(|point| {
                let (user_lat, user_lng) = current_gps_or_center.get();
                let dist_meters = haversine_distance_meters(user_lat, user_lng, point.latitude, point.longitude);
                let dist_str = format_distance(dist_meters);

                let (fill_pct, fill_color, fill_label) = match point.fill_level {
                    FillLevel::Low => (30, "bg-emerald-500", "Bajo (< 30% capacidad)"),
                    FillLevel::Medium => (65, "bg-amber-500", "Medio (50% - 70% capacidad)"),
                    FillLevel::Full => (98, "bg-red-500 animate-pulse", "Lleno (Capacidad Máxima)"),
                };

                let fly_to_point = {
                    let lat = point.latitude;
                    let lng = point.longitude;
                    move |_| {
                        let _ = js_sys::eval(&format!("window.ecoMap.flyTo({}, {}, 16.5);", lat, lng));
                    }
                };

                view! {
                    <div class="fixed inset-0 bg-slate-900/40 backdrop-blur-sm flex items-end sm:items-center justify-center p-0 sm:p-4 z-50 animate-in fade-in duration-150">
                        <div class="bg-white w-full sm:max-w-lg rounded-t-3xl sm:rounded-2xl border border-slate-200/90 p-6 shadow-[0_24px_64px_rgba(0,0,0,0.2)] max-h-[88vh] overflow-y-auto space-y-4">
                            // Top Drag handle for mobile
                            <div class="flex justify-center -mt-2 mb-1 sm:hidden">
                                <div class="w-12 h-1.5 bg-slate-200 rounded-full"></div>
                            </div>

                            // Referential Image Banner
                            {(!point.image_url.is_empty()).then(|| {
                                let img_src = point.image_url.clone();
                                view! {
                                    <div class="relative w-full h-44 rounded-2xl overflow-hidden bg-slate-100 border border-slate-200/60 shadow-inner">
                                        <img src=img_src class="w-full h-full object-cover" alt="Foto referencial del punto de acopio" />
                                        <div class="absolute top-2.5 right-2.5 px-2.5 py-1 rounded-full bg-slate-900/75 backdrop-blur-md text-[10px] font-semibold text-white flex items-center space-x-1.5 shadow-xs">
                                            <svg class="w-3 h-3 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                <path d="M14.5 4h-5L7 7H4a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-3l-2.5-3z"/>
                                                <circle cx="12" cy="13" r="3"/>
                                            </svg>
                                            <span>"Foto Referencial"</span>
                                        </div>
                                    </div>
                                }
                            })}

                            // Header row: Icon, Title & Distance
                            <div class="flex items-start justify-between gap-3 pt-1">
                                <div class="flex items-start space-x-3 min-w-0">
                                    <div class={format!("w-10 h-10 rounded-xl flex items-center justify-center shrink-0 border {}", point.waste_type.color_class())}>
                                        {match point.waste_type {
                                            WasteType::Recyclable => view! {
                                                <svg class="w-5 h-5 text-emerald-700" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                                                    <path d="M7 19H4.815a1.83 1.83 0 0 1-1.57-.881 1.785 1.785 0 0 1-.004-1.784L7.196 9.5"/>
                                                    <path d="M11 19h8.2a1.8 1.8 0 0 0 1.573-.896 1.79 1.79 0 0 0 0-1.792L16.8 9.5"/>
                                                    <path d="m15.5 13-3.5 6-3.5-6"/>
                                                    <path d="m2 9 3-5 3 5"/>
                                                    <path d="M14 4h5.5a1.8 1.8 0 0 1 1.57.88 1.79 1.79 0 0 1 .005 1.79L18.5 12"/>
                                                </svg>
                                            }.into_view(),
                                            WasteType::Organic => view! {
                                                <svg class="w-5 h-5 text-amber-700" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                                                    <path d="M11 20A7 7 0 0 1 9.8 6.1C15.5 5 17 4.48 19 2c1 2 2 4.18 2 8 0 5.5-4.78 10-10 10Z"/>
                                                    <path d="M2 21c0-3 1.85-5.36 5.08-6C9.5 14.52 12 13 13 12"/>
                                                </svg>
                                            }.into_view(),
                                            WasteType::General => view! {
                                                <svg class="w-5 h-5 text-slate-700" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                                                    <path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/>
                                                    <line x1="10" x2="10" y1="11" x2="17"/><line x1="14" x2="14" y1="11" x2="17"/>
                                                </svg>
                                            }.into_view(),
                                        }}
                                    </div>
                                    <div class="min-w-0">
                                        <h3 class="text-base font-bold text-slate-900 leading-snug">
                                            {point.name.clone()}
                                        </h3>
                                        {(!point.address.is_empty()).then(|| {
                                            let addr = point.address.clone();
                                            view! {
                                                <div class="flex items-center space-x-1.5 mt-1 text-slate-500 text-xs">
                                                    <svg class="w-3.5 h-3.5 shrink-0 text-slate-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                        <path d="M20 10c0 6-8 12-8 12s-8-6-8-12a8 8 0 0 1 16 0Z"/>
                                                        <circle cx="12" cy="10" r="3"/>
                                                    </svg>
                                                    <span class="truncate">{addr}</span>
                                                </div>
                                            }
                                        })}
                                    </div>
                                </div>

                                <div class="px-2.5 py-1 rounded-xl bg-sky-50 border border-sky-100 text-sky-700 flex flex-col items-center shrink-0">
                                    <span class="text-[9px] font-bold uppercase tracking-wider">"Distancia"</span>
                                    <span class="text-xs font-bold font-mono">{dist_str}</span>
                                </div>
                            </div>

                            // Description
                            <p class="text-xs text-slate-600 leading-relaxed">
                                {point.description.clone()}
                            </p>

                            <hr class="border-slate-100" />

                            // Waste Classification Card
                            <div class="space-y-1.5">
                                <span class="text-[10px] font-bold text-slate-400 uppercase tracking-wider">
                                    "Clasificación de Residuos Aceptados"
                                </span>
                                <div class={format!("p-3 rounded-xl border text-xs font-semibold leading-relaxed {}", point.waste_type.color_class())}>
                                    {point.waste_type.full_label()}
                                </div>
                            </div>

                            // Fill Level Gauge
                            <div class="space-y-1.5 p-3 rounded-xl bg-slate-50 border border-slate-200/80">
                                <div class="flex justify-between items-center text-xs">
                                    <span class="font-bold text-slate-700">"Nivel de Capacidad / Llenado:"</span>
                                    <span class={format!("px-2 py-0.5 rounded-full text-[10px] font-bold border {}", point.fill_level.badge_class())}>
                                        {fill_label}
                                    </span>
                                </div>
                                <div class="w-full bg-slate-200 rounded-full h-2 overflow-hidden">
                                    <div class={format!("h-2 rounded-full transition-all duration-500 {}", fill_color)} style=format!("width: {}%", fill_pct)></div>
                                </div>
                            </div>

                            // Citizen / Authority Fill Report Section
                            <div class="p-3 bg-slate-50 border border-slate-200 rounded-xl space-y-2">
                                <div class="flex items-center justify-between">
                                    <span class="text-[11px] font-bold text-slate-700">
                                        "¿Estás en el lugar? Actualiza el estado del contenedor:"
                                    </span>
                                    {move || is_updating_fill.get().then(|| view! {
                                        <span class="inline-block w-3.5 h-3.5 border-2 border-slate-400 border-t-slate-800 rounded-full animate-spin"></span>
                                    })}
                                </div>

                                {move || fill_report_msg.get().map(|(is_ok, msg)| view! {
                                    <div class={format!("p-2 rounded-lg text-[11px] font-medium flex items-center space-x-1.5 {}", if is_ok { "bg-emerald-50 text-emerald-800 border border-emerald-200" } else { "bg-amber-50 text-amber-800 border border-amber-200" })}>
                                        <span>{if is_ok { "✅" } else { "ℹ️" }}</span>
                                        <span>{msg}</span>
                                    </div>
                                })}

                                <div class="grid grid-cols-3 gap-2 pt-0.5">
                                    <button
                                        class=move || {
                                            if point.fill_level == FillLevel::Low {
                                                "py-1.5 px-2 rounded-xl text-xs font-bold bg-emerald-600 text-white shadow-xs transition"
                                            } else {
                                                "py-1.5 px-2 rounded-xl text-xs font-semibold bg-white hover:bg-emerald-50 text-emerald-800 border border-emerald-200 transition"
                                            }
                                        }
                                        disabled=move || is_updating_fill.get()
                                        on:click={
                                            let pid = point.id_string();
                                            move |_| report_fill_level(pid.clone(), FillLevel::Low)
                                        }
                                    >
                                        "🟢 Bajo"
                                    </button>
                                    <button
                                        class=move || {
                                            if point.fill_level == FillLevel::Medium {
                                                "py-1.5 px-2 rounded-xl text-xs font-bold bg-amber-600 text-white shadow-xs transition"
                                            } else {
                                                "py-1.5 px-2 rounded-xl text-xs font-semibold bg-white hover:bg-amber-50 text-amber-800 border border-amber-200 transition"
                                            }
                                        }
                                        disabled=move || is_updating_fill.get()
                                        on:click={
                                            let pid = point.id_string();
                                            move |_| report_fill_level(pid.clone(), FillLevel::Medium)
                                        }
                                    >
                                        "🟡 Medio"
                                    </button>
                                    <button
                                        class=move || {
                                            if point.fill_level == FillLevel::Full {
                                                "py-1.5 px-2 rounded-xl text-xs font-bold bg-red-600 text-white shadow-xs animate-pulse transition"
                                            } else {
                                                "py-1.5 px-2 rounded-xl text-xs font-semibold bg-white hover:bg-red-50 text-red-800 border border-red-200 transition"
                                            }
                                        }
                                        disabled=move || is_updating_fill.get()
                                        on:click={
                                            let pid = point.id_string();
                                            move |_| report_fill_level(pid.clone(), FillLevel::Full)
                                        }
                                    >
                                        "🔴 Lleno"
                                    </button>
                                </div>
                            </div>

                            // Truck Schedule Card
                            <div class="p-3 bg-blue-50/70 border border-blue-200/70 rounded-xl flex items-start space-x-3">
                                <div class="w-8 h-8 rounded-lg bg-blue-600 text-white flex items-center justify-center shrink-0 shadow-xs">
                                    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                        <path d="M10 17h4V5H2v12h3m11 0h3l3-4v-5h-6v9z"/>
                                        <circle cx="7.5" cy="17.5" r="2.5"/>
                                        <circle cx="17.5" cy="17.5" r="2.5"/>
                                    </svg>
                                </div>
                                <div class="min-w-0">
                                    <p class="text-[10px] font-bold text-blue-800 uppercase tracking-wider">
                                        "Paso del Camión Recolector Municipal"
                                    </p>
                                    <p class="text-xs font-semibold text-blue-950 mt-0.5">
                                        {point.truck_schedule.clone()}
                                    </p>
                                </div>
                            </div>

                            // Recommendations
                            {(!point.recommendations.is_empty()).then(|| {
                                let rec = point.recommendations.clone();
                                view! {
                                    <div class="p-3 bg-amber-50/80 border border-amber-200/80 rounded-xl flex items-start space-x-3">
                                        <svg class="w-4 h-4 text-amber-700 mt-0.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                            <circle cx="12" cy="12" r="10"></circle>
                                            <line x1="12" y1="8" x2="12" y2="12"></line>
                                            <line x1="12" y1="16" x2="12.01" y2="16"></line>
                                        </svg>
                                        <div class="min-w-0">
                                            <p class="text-[10px] font-bold text-amber-900 uppercase tracking-wider">
                                                "Recomendación Ciudadana"
                                            </p>
                                            <p class="text-xs text-amber-900 leading-snug mt-0.5">
                                                {rec}
                                            </p>
                                        </div>
                                    </div>
                                }
                            })}

                            // Footer Buttons
                            <div class="flex justify-end space-x-2 pt-2 border-t border-slate-100">
                                <button
                                    class="px-3.5 py-2 text-xs font-semibold text-slate-700 bg-slate-100 hover:bg-slate-200 rounded-xl transition flex items-center space-x-1.5"
                                    on:click=fly_to_point
                                >
                                    <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                        <circle cx="12" cy="12" r="10"/>
                                        <polygon points="16.24 7.76 14.12 14.12 7.76 16.24 9.88 9.88 16.24 7.76"/>
                                    </svg>
                                    <span>"Ver en Mapa"</span>
                                </button>
                                <button
                                    class="px-4 py-2 text-xs font-semibold bg-slate-900 hover:bg-slate-800 text-white rounded-xl shadow-xs transition"
                                    on:click=move |_| selected_waste_point_id.set(None)
                                >
                                    "Cerrar"
                                </button>
                            </div>
                        </div>
                    </div>
                }
            })
        }}
    }
}
