//! Editor visual de rutas de recolección (solo autoridad).
//!
//! La geometría (puntos de control, trazo ajustado a calles y paradas) vive en
//! `window.ecoRouteEditor` (index.html). Este componente maneja el formulario,
//! la lista de paradas y el guardado contra `/api/routes/`.

use leptos::*;
use wasm_bindgen::prelude::*;

use crate::api::{route_detail_url, routes_url};
use crate::models::{AuthUser, CollectionRoute, FleetTruck, RouteEditorState, WastePoint};

const MAP_ELEMENT_ID: &str = "eco-route-editor-map";

const DISTRICTS: &[(&str, &str)] = &[
    ("yanacancha", "Yanacancha"),
    ("chaupimarca", "Chaupimarca"),
    ("simonBolivar", "Simón Bolívar"),
];

const WASTE_TYPES: &[(&str, &str)] = &[
    ("general", "General"),
    ("organic", "Orgánico"),
    ("recyclable", "Reciclable"),
];

fn js_call(script: String) {
    let _ = js_sys::eval(&script);
}

fn editor_call(method: &str, args: &[serde_json::Value]) {
    let args_js = args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ");
    js_call(format!("window.ecoRouteEditor && window.ecoRouteEditor.{}({});", method, args_js));
}

fn format_km(m: Option<f64>) -> String {
    match m {
        Some(m) if m >= 1000.0 => format!("{:.2} km", m / 1000.0),
        Some(m) => format!("{:.0} m", m),
        None => "—".into(),
    }
}

#[component]
pub fn RouteEditor(
    route: Option<CollectionRoute>,
    trucks: Vec<FleetTruck>,
    waste_points: RwSignal<Vec<WastePoint>>,
    current_user: RwSignal<Option<AuthUser>>,
    #[prop(into)] on_close: Callback<bool>,
) -> impl IntoView {
    let route_id = route.as_ref().map(|r| r.id_string());
    let is_new = route_id.is_none();
    let base = route.clone().unwrap_or(CollectionRoute {
        id: serde_json::Value::Null,
        name: String::new(),
        description: String::new(),
        district: "yanacancha".into(),
        district_display: String::new(),
        waste_type: "general".into(),
        color: "#16a34a".into(),
        schedule: String::new(),
        path: vec![],
        default_truck_code: None,
        stops: vec![],
        waypoints: vec![],
        snap_to_roads: true,
        default_truck: None,
        is_active: true,
    });

    let name = create_rw_signal(base.name.clone());
    let description = create_rw_signal(base.description.clone());
    let district = create_rw_signal(if base.district.is_empty() { "yanacancha".to_string() } else { base.district.clone() });
    let waste_type = create_rw_signal(if base.waste_type.is_empty() { "general".to_string() } else { base.waste_type.clone() });
    let color = create_rw_signal(base.color.clone());
    let schedule = create_rw_signal(base.schedule.clone());
    let default_truck = create_rw_signal(base.default_truck);
    let is_active = create_rw_signal(base.is_active);
    let snap = create_rw_signal(base.snap_to_roads);
    let eta_speed = create_rw_signal(12.0f64);
    let eta_dwell = create_rw_signal(2.0f64);

    let state = create_rw_signal(RouteEditorState::default());
    let saving = create_rw_signal(false);
    let save_error = create_rw_signal(None::<String>);

    // JS → Rust
    {
        let on_change = Closure::<dyn FnMut(String)>::new(move |raw: String| {
            if let Ok(st) = serde_json::from_str::<RouteEditorState>(&raw) {
                state.set(st);
            }
        });
        if let Some(win) = web_sys::window() {
            let _ = js_sys::Reflect::set(&win, &JsValue::from_str("onRouteEditorChange"), on_change.as_ref().unchecked_ref());
        }
        on_change.forget();
    }

    // Inicializar el mapa del editor cuando el contenedor ya está en el DOM
    {
        let token = current_user.with_untracked(|u| u.as_ref().map(|u| u.token.clone())).unwrap_or_default();
        let wps: Vec<serde_json::Value> = waste_points
            .get_untracked()
            .iter()
            .map(|w| serde_json::json!({ "id": w.id_string(), "name": w.name, "latitude": w.latitude, "longitude": w.longitude }))
            .collect();
        let opts = serde_json::json!({
            "token": token,
            "color": base.color,
            "snap": base.snap_to_roads,
            "path": base.path,
            "waypoints": base.waypoints,
            "stops": base.stops,
            "waste_points": wps,
        });
        set_timeout(
            move || editor_call("init", &[serde_json::json!(MAP_ELEMENT_ID), opts]),
            std::time::Duration::from_millis(60),
        );
    }
    on_cleanup(|| js_call("window.ecoRouteEditor && window.ecoRouteEditor.destroy();".into()));

    let close = move |saved: bool| {
        js_call("window.ecoRouteEditor && window.ecoRouteEditor.destroy();".into());
        on_close.call(saved);
    };

    let save = move |_| {
        let st = state.get_untracked();
        let n = name.get_untracked().trim().to_string();
        if n.is_empty() {
            save_error.set(Some("Ponle un nombre a la ruta.".into()));
            return;
        }
        if st.path.len() < 2 {
            save_error.set(Some("Marca al menos 2 puntos sobre el mapa para trazar la ruta.".into()));
            return;
        }
        if st.snapping {
            save_error.set(Some("Espera a que termine el ajuste a calles.".into()));
            return;
        }
        let stops: Vec<serde_json::Value> = st
            .stops
            .iter()
            .enumerate()
            .map(|(i, s)| {
                serde_json::json!({
                    "order": i,
                    "name": if s.name.trim().is_empty() { format!("Parada {}", i + 1) } else { s.name.trim().to_string() },
                    "latitude": s.latitude,
                    "longitude": s.longitude,
                    "waste_point": s.waste_point,
                    "eta_offset_minutes": s.eta_offset_minutes,
                })
            })
            .collect();
        let payload = serde_json::json!({
            "name": n,
            "description": description.get_untracked(),
            "district": district.get_untracked(),
            "waste_type": waste_type.get_untracked(),
            "color": color.get_untracked(),
            "schedule": schedule.get_untracked(),
            "path": st.path,
            "waypoints": st.waypoints,
            "snap_to_roads": st.snap,
            "default_truck": default_truck.get_untracked(),
            "is_active": is_active.get_untracked(),
            "stops": stops,
        });
        let Some(tok) = current_user.with_untracked(|u| u.as_ref().map(|u| u.token.clone())) else { return };
        let url = route_id.clone().map(|id| route_detail_url(&id)).unwrap_or_else(routes_url);
        let is_update = route_id.is_some();
        saving.set(true);
        save_error.set(None);
        spawn_local(async move {
            let req = if is_update {
                gloo_net::http::Request::put(&url)
            } else {
                gloo_net::http::Request::post(&url)
            }
            .header("Authorization", &format!("Token {}", tok))
            .json(&payload);
            let resp = match req {
                Ok(r) => r.send().await,
                Err(e) => Err(e),
            };
            saving.set(false);
            match resp {
                Ok(r) if r.ok() => close(true),
                Ok(r) => {
                    let body = r.json::<serde_json::Value>().await.unwrap_or_default();
                    // Errores de DRF por campo: {"path": ["..."], ...}
                    let msg = body
                        .as_object()
                        .and_then(|o| o.iter().next())
                        .map(|(k, v)| {
                            let detail = v.as_array().and_then(|a| a.first()).and_then(|x| x.as_str()).or(v.as_str()).unwrap_or("valor inválido");
                            format!("{}: {}", k, detail)
                        })
                        .unwrap_or_else(|| format!("Error HTTP {}", r.status()));
                    save_error.set(Some(msg));
                }
                Err(_) => save_error.set(Some("Sin conexión con el servidor.".into())),
            }
        });
    };

    let input_class = "w-full rounded-lg border border-slate-200 bg-white px-2.5 py-1.5 text-xs text-slate-800 focus:outline-none focus:ring-2 focus:ring-emerald-500";
    let label_class = "text-[10px] font-semibold text-slate-500 uppercase tracking-wide";
    let mode_btn = move |m: &'static str| {
        move || {
            if state.with(|s| s.mode == m || (s.mode.is_empty() && m == "route")) {
                "flex-1 px-2 py-1.5 rounded-lg text-[11px] font-bold bg-slate-900 text-white"
            } else {
                "flex-1 px-2 py-1.5 rounded-lg text-[11px] font-semibold text-slate-600 hover:bg-slate-100"
            }
        }
    };

    view! {
        <div class="fixed inset-0 z-[60] bg-slate-900/60 backdrop-blur-sm flex items-stretch justify-center sm:p-4">
            <div class="bg-white w-full max-w-7xl sm:rounded-2xl shadow-2xl flex flex-col overflow-hidden">
                // Encabezado
                <div class="flex items-center justify-between px-4 py-3 border-b border-slate-200">
                    <div>
                        <h3 class="text-sm font-bold text-slate-900">
                            {if is_new { "Nueva ruta de recolección" } else { "Editar ruta de recolección" }}
                        </h3>
                        <p class="text-[11px] text-slate-500">
                            "Haz clic en el mapa para marcar el recorrido. Arrastra los puntos para corregirlo."
                        </p>
                    </div>
                    <div class="flex items-center gap-2">
                        <button class="px-3 py-1.5 rounded-lg text-xs font-semibold text-slate-600 hover:bg-slate-100" on:click=move |_| close(false)>
                            "Cancelar"
                        </button>
                        <button
                            class="px-4 py-1.5 rounded-lg text-xs font-bold bg-emerald-600 hover:bg-emerald-700 disabled:opacity-60 text-white shadow"
                            disabled=move || saving.get() || state.with(|s| s.snapping)
                            on:click=save
                        >
                            {move || if saving.get() { "Guardando…" } else { "Guardar ruta" }}
                        </button>
                    </div>
                </div>

                <div class="flex-1 flex flex-col-reverse lg:flex-row min-h-0">
                    // Panel lateral
                    <aside class="lg:w-[360px] shrink-0 border-t lg:border-t-0 lg:border-r border-slate-200 overflow-y-auto p-4 space-y-4 max-h-[45vh] lg:max-h-none">
                        {move || save_error.get().map(|e| view! {
                            <div class="rounded-lg bg-red-50 border border-red-200 text-red-700 text-xs px-3 py-2">{e}</div>
                        })}

                        <div class="space-y-2.5">
                            <label class="block space-y-1">
                                <span class=label_class>"Nombre"</span>
                                <input class=input_class placeholder="Ej. Ruta Centro Yanacancha"
                                    prop:value=move || name.get()
                                    on:input=move |ev| name.set(event_target_value(&ev)) />
                            </label>
                            <div class="grid grid-cols-2 gap-2">
                                <label class="block space-y-1">
                                    <span class=label_class>"Distrito"</span>
                                    <select class=input_class on:change=move |ev| district.set(event_target_value(&ev))>
                                        {DISTRICTS.iter().map(|(v, l)| view! {
                                            <option value=*v selected=move || district.get() == *v>{*l}</option>
                                        }).collect_view()}
                                    </select>
                                </label>
                                <label class="block space-y-1">
                                    <span class=label_class>"Residuo"</span>
                                    <select class=input_class on:change=move |ev| waste_type.set(event_target_value(&ev))>
                                        {WASTE_TYPES.iter().map(|(v, l)| view! {
                                            <option value=*v selected=move || waste_type.get() == *v>{*l}</option>
                                        }).collect_view()}
                                    </select>
                                </label>
                            </div>
                            <label class="block space-y-1">
                                <span class=label_class>"Días y horario"</span>
                                <input class=input_class placeholder="Lunes, Miércoles y Viernes 06:00 - 09:00"
                                    prop:value=move || schedule.get()
                                    on:input=move |ev| schedule.set(event_target_value(&ev)) />
                            </label>
                            <div class="grid grid-cols-[1fr_auto] gap-2 items-end">
                                <label class="block space-y-1">
                                    <span class=label_class>"Camión habitual"</span>
                                    <select class=input_class on:change=move |ev| default_truck.set(event_target_value(&ev).parse::<i64>().ok())>
                                        <option value="" selected=move || default_truck.get().is_none()>"Sin asignar"</option>
                                        {trucks.iter().map(|t| {
                                            let id = t.id;
                                            let label = format!("{} · {}", t.code, t.plate);
                                            view! { <option value=id.to_string() selected=move || default_truck.get() == Some(id)>{label}</option> }
                                        }).collect_view()}
                                    </select>
                                </label>
                                <label class="block space-y-1">
                                    <span class=label_class>"Color"</span>
                                    <input type="color" class="h-[30px] w-12 rounded-lg border border-slate-200 cursor-pointer"
                                        prop:value=move || color.get()
                                        on:input=move |ev| {
                                            let c = event_target_value(&ev);
                                            editor_call("setColor", &[serde_json::json!(c)]);
                                            color.set(c);
                                        } />
                                </label>
                            </div>
                            <label class="block space-y-1">
                                <span class=label_class>"Descripción"</span>
                                <textarea class=input_class rows="2"
                                    prop:value=move || description.get()
                                    on:input=move |ev| description.set(event_target_value(&ev))></textarea>
                            </label>
                            <label class="flex items-center gap-2 text-xs text-slate-700 cursor-pointer">
                                <input type="checkbox" prop:checked=move || is_active.get()
                                    on:change=move |ev| is_active.set(event_target_checked(&ev)) />
                                <span>"Publicada (visible para la ciudadanía)"</span>
                            </label>
                        </div>

                        // Paradas
                        <div class="space-y-2 pt-3 border-t border-slate-100">
                            <div class="flex items-center justify-between">
                                <span class="text-xs font-bold text-slate-800">
                                    {move || format!("Paradas ({})", state.with(|s| s.stops.len()))}
                                </span>
                                <button class="text-[11px] font-semibold text-emerald-700 hover:underline"
                                    on:click=move |_| editor_call("setMode", &[serde_json::json!("stops")])>
                                    "+ Añadir en el mapa"
                                </button>
                            </div>
                            <div class="flex items-end gap-2 text-[11px] text-slate-600">
                                <label class="space-y-0.5">
                                    <span class="block">"Velocidad (km/h)"</span>
                                    <input type="number" min="3" max="60" class="w-16 rounded border border-slate-200 px-1.5 py-1"
                                        prop:value=move || eta_speed.get().to_string()
                                        on:input=move |ev| eta_speed.set(event_target_value(&ev).parse().unwrap_or(12.0)) />
                                </label>
                                <label class="space-y-0.5">
                                    <span class="block">"Min. por parada"</span>
                                    <input type="number" min="0" max="30" class="w-16 rounded border border-slate-200 px-1.5 py-1"
                                        prop:value=move || eta_dwell.get().to_string()
                                        on:input=move |ev| eta_dwell.set(event_target_value(&ev).parse().unwrap_or(2.0)) />
                                </label>
                                <button class="px-2 py-1 rounded border border-slate-200 font-semibold hover:bg-slate-50"
                                    title="Ordena las paradas según el trazo y estima los minutos desde el inicio"
                                    on:click=move |_| editor_call("computeEtas", &[serde_json::json!(eta_speed.get_untracked()), serde_json::json!(eta_dwell.get_untracked())])>
                                    "⏱ Calcular tiempos"
                                </button>
                            </div>
                            {move || {
                                let stops = state.with(|s| s.stops.clone());
                                if stops.is_empty() {
                                    return view! {
                                        <p class="text-[11px] text-slate-500">"Activa el modo Paradas y haz clic en el mapa o en un punto de acopio (círculos grises)."</p>
                                    }.into_view();
                                }
                                let total = stops.len();
                                stops.into_iter().enumerate().map(|(i, s)| {
                                    let linked = s.waste_point.is_some();
                                    view! {
                                        <div class="flex items-center gap-1.5 rounded-lg border border-slate-200 px-2 py-1.5">
                                            <span class=if linked { "w-5 h-5 shrink-0 rounded-full bg-teal-700 text-white text-[10px] font-bold flex items-center justify-center" } else { "w-5 h-5 shrink-0 rounded-full bg-slate-800 text-white text-[10px] font-bold flex items-center justify-center" }>
                                                {i + 1}
                                            </span>
                                            <input class="flex-1 min-w-0 text-xs border-0 focus:ring-0 p-0" prop:value=s.name.clone()
                                                on:change=move |ev| editor_call("updateStop", &[serde_json::json!(i), serde_json::json!({ "name": event_target_value(&ev) })]) />
                                            <input type="number" min="0" class="w-12 text-[11px] rounded border border-slate-200 px-1 py-0.5" title="Minutos desde el inicio"
                                                prop:value=s.eta_offset_minutes.to_string()
                                                on:change=move |ev| editor_call("updateStop", &[serde_json::json!(i), serde_json::json!({ "eta_offset_minutes": event_target_value(&ev).parse::<u32>().unwrap_or(0) })]) />
                                            <span class="text-[10px] text-slate-400">"min"</span>
                                            <button class="text-slate-400 hover:text-slate-700 disabled:opacity-30 px-0.5" disabled=i == 0 title="Subir"
                                                on:click=move |_| editor_call("moveStop", &[serde_json::json!(i), serde_json::json!(-1)])>"↑"</button>
                                            <button class="text-slate-400 hover:text-slate-700 disabled:opacity-30 px-0.5" disabled=i + 1 == total title="Bajar"
                                                on:click=move |_| editor_call("moveStop", &[serde_json::json!(i), serde_json::json!(1)])>"↓"</button>
                                            <button class="text-red-400 hover:text-red-600 px-0.5" title="Quitar parada"
                                                on:click=move |_| editor_call("removeStop", &[serde_json::json!(i)])>"✕"</button>
                                        </div>
                                    }
                                }).collect_view()
                            }}
                        </div>
                    </aside>

                    // Mapa + barra de herramientas
                    <section class="flex-1 relative min-h-[50vh]">
                        <div id=MAP_ELEMENT_ID class="absolute inset-0 z-0"></div>

                        <div class="absolute top-3 left-14 right-3 z-[500] flex flex-wrap items-center gap-2 pointer-events-none">
                            <div class="pointer-events-auto bg-white/95 rounded-xl shadow border border-slate-200 p-1 flex gap-1 w-52">
                                <button class=mode_btn("route") on:click=move |_| editor_call("setMode", &[serde_json::json!("route")])>"✏️ Trazo"</button>
                                <button class=mode_btn("stops") on:click=move |_| editor_call("setMode", &[serde_json::json!("stops")])>"📍 Paradas"</button>
                            </div>
                            <label class="pointer-events-auto bg-white/95 rounded-xl shadow border border-slate-200 px-3 py-1.5 flex items-center gap-2 text-[11px] font-semibold text-slate-700 cursor-pointer">
                                <input type="checkbox" prop:checked=move || snap.get()
                                    on:change=move |ev| {
                                        let on = event_target_checked(&ev);
                                        snap.set(on);
                                        editor_call("setSnap", &[serde_json::json!(on)]);
                                    } />
                                <span>"🛣️ Ajustar a calles"</span>
                            </label>
                            <div class="pointer-events-auto bg-white/95 rounded-xl shadow border border-slate-200 p-1 flex gap-1 text-[11px] font-semibold text-slate-700">
                                <button class="px-2 py-1 rounded-lg hover:bg-slate-100 disabled:opacity-40" disabled=move || !state.with(|s| s.can_undo)
                                    on:click=move |_| editor_call("undo", &[])>"↶ Deshacer"</button>
                                <button class="px-2 py-1 rounded-lg hover:bg-slate-100" on:click=move |_| editor_call("reverse", &[])>"⇄ Invertir"</button>
                                <button class="px-2 py-1 rounded-lg hover:bg-slate-100" on:click=move |_| editor_call("fit", &[])>"⤢ Encuadrar"</button>
                                <button class="px-2 py-1 rounded-lg hover:bg-red-50 text-red-600"
                                    on:click=move |_| {
                                        let ok = web_sys::window().and_then(|w| w.confirm_with_message("¿Borrar todo el trazo? Las paradas se conservan.").ok()).unwrap_or(false);
                                        if ok { editor_call("clearRoute", &[]); }
                                    }>"🗑 Borrar trazo"</button>
                            </div>
                        </div>

                        <div class="absolute bottom-3 left-3 z-[500] bg-white/95 rounded-xl shadow border border-slate-200 px-3 py-2 text-[11px] text-slate-700 space-y-0.5 max-w-[calc(100%-1.5rem)]">
                            {move || {
                                let st = state.get();
                                let mins = st.duration_s.map(|d| format!(" · ~{:.0} min en vehículo", d / 60.0)).unwrap_or_default();
                                view! {
                                    <p class="font-semibold">
                                        {format!("{} puntos de control · {}{}", st.waypoints.len(), format_km(st.distance_m), mins)}
                                    </p>
                                    <p class="text-slate-500">
                                        {if st.mode == "stops" {
                                            "Modo Paradas: clic en el mapa o en un punto de acopio para añadir una parada."
                                        } else {
                                            "Modo Trazo: clic para añadir al final · clic sobre la línea para insertar · doble clic en un punto para borrarlo."
                                        }}
                                    </p>
                                    {st.snapping.then(|| view! { <p class="text-sky-700">"🛣️ Ajustando a calles…"</p> })}
                                    {st.error.clone().map(|e| view! { <p class="text-amber-700">{e}</p> })}
                                }
                            }}
                        </div>
                    </section>
                </div>
            </div>
        </div>
    }
}
