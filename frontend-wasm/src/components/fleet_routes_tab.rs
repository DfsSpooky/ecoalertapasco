//! Pestaña "Rutas y Flota" del panel de autoridades:
//! camiones en recorrido en vivo, gestión de rutas (con editor visual) y flota.

use leptos::*;

use crate::api::{admin_url, route_detail_url, routes_url, run_action_url, trucks_url};
use crate::components::route_editor::RouteEditor;
use crate::models::{AuthUser, CollectionRoute, FleetTruck, TruckRun, WastePoint};

fn ago_from_iso(iso: &Option<String>, now_ms: f64) -> (String, bool) {
    let Some(ts) = iso.as_ref().map(|s| js_sys::Date::parse(s)).filter(|t| !t.is_nan()) else {
        return ("sin posición".into(), true);
    };
    let s = ((now_ms - ts) / 1000.0).max(0.0) as i64;
    let label = if s < 60 { format!("hace {} s", s) } else if s < 3600 { format!("hace {} min", s / 60) } else { format!("hace {} h", s / 3600) };
    (label, s > 180)
}

fn waste_label(t: &str) -> &'static str {
    match t {
        "organic" => "Orgánico",
        "recyclable" => "Reciclable",
        _ => "General",
    }
}

#[component]
pub fn FleetRoutesTab(
    truck_runs: RwSignal<Vec<TruckRun>>,
    waste_points: RwSignal<Vec<WastePoint>>,
    current_user: RwSignal<Option<AuthUser>>,
    show_dashboard: RwSignal<bool>,
) -> impl IntoView {
    let routes = create_rw_signal(Vec::<CollectionRoute>::new());
    let trucks = create_rw_signal(Vec::<FleetTruck>::new());
    let loading = create_rw_signal(false);
    let message = create_rw_signal(None::<(bool, String)>);
    // Some(None) = nueva ruta; Some(Some(r)) = editar r
    let editing = create_rw_signal(None::<Option<CollectionRoute>>);
    let now_ms = create_rw_signal(js_sys::Date::now());
    let _ = set_interval_with_handle(move || now_ms.set(js_sys::Date::now()), std::time::Duration::from_secs(5));

    let token = move || current_user.with_untracked(|u| u.as_ref().map(|u| u.token.clone()));

    let reload = move || {
        let Some(tok) = token() else { return };
        loading.set(true);
        spawn_local(async move {
            let auth = format!("Token {}", tok);
            if let Ok(r) = gloo_net::http::Request::get(&routes_url()).header("Authorization", &auth).send().await {
                if let Ok(list) = r.json::<Vec<CollectionRoute>>().await {
                    routes.set(list);
                }
            }
            if let Ok(r) = gloo_net::http::Request::get(&trucks_url()).header("Authorization", &auth).send().await {
                if let Ok(list) = r.json::<Vec<FleetTruck>>().await {
                    trucks.set(list);
                }
            }
            loading.set(false);
        });
    };
    reload();

    let api_send = move |method: &'static str, url: String, body: Option<serde_json::Value>, ok_msg: String| {
        let Some(tok) = token() else { return };
        spawn_local(async move {
            let builder = match method {
                "DELETE" => gloo_net::http::Request::delete(&url),
                "PATCH" => gloo_net::http::Request::patch(&url),
                _ => gloo_net::http::Request::post(&url),
            }
            .header("Authorization", &format!("Token {}", tok));
            let resp = match body {
                Some(b) => match builder.json(&b) {
                    Ok(r) => r.send().await,
                    Err(e) => Err(e),
                },
                None => builder.send().await,
            };
            match resp {
                Ok(r) if r.ok() => message.set(Some((true, ok_msg))),
                Ok(r) => {
                    let b = r.json::<serde_json::Value>().await.unwrap_or_default();
                    let err = b.get("error").or(b.get("detail")).and_then(|v| v.as_str()).map(str::to_string)
                        .unwrap_or_else(|| format!("Error HTTP {}", r.status()));
                    message.set(Some((false, err)));
                }
                Err(_) => message.set(Some((false, "Sin conexión con el servidor.".into()))),
            }
            reload();
        });
    };

    let focus_truck = move |lat: f64, lng: f64| {
        show_dashboard.set(false);
        let _ = js_sys::eval(&format!(
            "setTimeout(() => window.ecoMap && window.ecoMap.map && window.ecoMap.map.setView([{}, {}], 17), 150);",
            lat, lng
        ));
    };

    let card = "rounded-2xl bg-white border border-slate-200 shadow-xs";

    view! {
        <div class="space-y-6 animate-in fade-in duration-150">
            {move || message.get().map(|(ok, m)| view! {
                <div class=if ok { "rounded-xl bg-emerald-50 border border-emerald-200 text-emerald-800 text-xs px-3 py-2 flex justify-between" } else { "rounded-xl bg-red-50 border border-red-200 text-red-700 text-xs px-3 py-2 flex justify-between" }>
                    <span>{m}</span>
                    <button class="opacity-60 hover:opacity-100" on:click=move |_| message.set(None)>"✕"</button>
                </div>
            })}

            // KPIs
            <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
                <div class=format!("p-3.5 {}", card)>
                    <p class="text-xs font-medium text-slate-500">"Camiones en recorrido"</p>
                    <h3 class="text-xl font-bold text-emerald-700 font-mono mt-0.5">{move || truck_runs.with(|r| r.iter().filter(|x| x.status == "active").count())}</h3>
                    <p class="text-[10px] text-slate-400">"Compartiendo GPS ahora"</p>
                </div>
                <div class=format!("p-3.5 {}", card)>
                    <p class="text-xs font-medium text-slate-500">"En pausa / sin señal"</p>
                    <h3 class="text-xl font-bold text-amber-600 font-mono mt-0.5">
                        {move || { let now = now_ms.get(); truck_runs.with(|r| r.iter().filter(|x| x.status == "paused" || ago_from_iso(&x.last_position_at, now).1).count()) }}
                    </h3>
                    <p class="text-[10px] text-slate-400">"Requieren atención"</p>
                </div>
                <div class=format!("p-3.5 {}", card)>
                    <p class="text-xs font-medium text-slate-500">"Rutas publicadas"</p>
                    <h3 class="text-xl font-bold text-slate-900 font-mono mt-0.5">{move || routes.with(|r| r.iter().filter(|x| x.is_active).count())}</h3>
                    <p class="text-[10px] text-slate-400">{move || format!("{} en total", routes.with(|r| r.len()))}</p>
                </div>
                <div class=format!("p-3.5 {}", card)>
                    <p class="text-xs font-medium text-slate-500">"Camiones operativos"</p>
                    <h3 class="text-xl font-bold text-slate-900 font-mono mt-0.5">{move || trucks.with(|t| t.iter().filter(|x| x.is_active).count())}</h3>
                    <p class="text-[10px] text-slate-400">{move || format!("{} sin conductor", trucks.with(|t| t.iter().filter(|x| x.driver_username.is_none()).count()))}</p>
                </div>
            </div>

            // Flota en vivo
            <section class=format!("{} p-4 space-y-3", card)>
                <div class="flex items-center justify-between">
                    <h3 class="text-sm font-bold text-slate-900">"🚛 Flota en vivo"</h3>
                    <span class="text-[11px] text-slate-500">"Se actualiza en tiempo real"</span>
                </div>
                {move || {
                    let runs = truck_runs.get();
                    let now = now_ms.get();
                    if runs.is_empty() {
                        return view! { <p class="text-xs text-slate-500">"Ningún camión está en recorrido en este momento."</p> }.into_view();
                    }
                    view! {
                        <div class="grid sm:grid-cols-2 lg:grid-cols-3 gap-3">
                            {runs.into_iter().map(|run| {
                                let (ago, stale) = ago_from_iso(&run.last_position_at, now);
                                let (badge, badge_cls) = if run.status == "paused" {
                                    ("⏸ En pausa", "bg-amber-100 text-amber-800")
                                } else if stale {
                                    ("📡 Sin señal", "bg-red-100 text-red-700")
                                } else {
                                    ("● En recorrido", "bg-emerald-100 text-emerald-800")
                                };
                                let pos = run.last_latitude.zip(run.last_longitude);
                                let run_id = run.id;
                                let code = run.truck_code.clone();
                                let color = if run.route_color.is_empty() { "#16a34a".to_string() } else { run.route_color.clone() };
                                view! {
                                    <div class="rounded-xl border border-slate-200 p-3 space-y-2" style=format!("border-left: 4px solid {}", color)>
                                        <div class="flex items-start justify-between gap-2">
                                            <div>
                                                <p class="text-sm font-bold text-slate-900">{format!("Camión {}", run.truck_code)}</p>
                                                <p class="text-[11px] text-slate-500">{run.route_name.clone()}</p>
                                            </div>
                                            <span class=format!("text-[10px] font-bold px-2 py-0.5 rounded-full {}", badge_cls)>{badge}</span>
                                        </div>
                                        <p class="text-[11px] text-slate-600">
                                            {format!("Última posición {}", ago)}
                                            {run.last_speed_kmh.filter(|_| run.status == "active").map(|v| format!(" · {:.0} km/h", v))}
                                        </p>
                                        <div class="flex gap-2">
                                            <button class="flex-1 px-2 py-1.5 rounded-lg text-[11px] font-semibold border border-slate-200 hover:bg-slate-50 disabled:opacity-40"
                                                disabled=pos.is_none()
                                                on:click=move |_| if let Some((lat, lng)) = pos { focus_truck(lat, lng) }>
                                                "📍 Ver en mapa"
                                            </button>
                                            <button class="flex-1 px-2 py-1.5 rounded-lg text-[11px] font-semibold bg-slate-900 text-white hover:bg-black"
                                                on:click=move |_| {
                                                    let ok = web_sys::window().and_then(|w| w.confirm_with_message(&format!("¿Finalizar el recorrido del camión {}? Dejará de mostrarse a la ciudadanía.", code)).ok()).unwrap_or(false);
                                                    if ok {
                                                        api_send("POST", run_action_url(run_id, "finish"), None, "Recorrido finalizado.".into());
                                                    }
                                                }>
                                                "■ Finalizar"
                                            </button>
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_view()
                }}
            </section>

            // Rutas
            <section class=format!("{} p-4 space-y-3", card)>
                <div class="flex items-center justify-between gap-2">
                    <div>
                        <h3 class="text-sm font-bold text-slate-900">"🗺️ Rutas de recolección"</h3>
                        <p class="text-[11px] text-slate-500">"Las rutas publicadas aparecen al instante en el mapa ciudadano."</p>
                    </div>
                    <button class="px-3 py-2 rounded-xl text-xs font-bold bg-emerald-600 hover:bg-emerald-700 text-white shadow"
                        on:click=move |_| editing.set(Some(None))>
                        "+ Nueva ruta"
                    </button>
                </div>
                {move || {
                    let list = routes.get();
                    if list.is_empty() {
                        return view! {
                            <p class="text-xs text-slate-500 py-4 text-center">
                                {move || if loading.get() { "Cargando rutas…" } else { "Aún no hay rutas. Crea la primera con “Nueva ruta”." }}
                            </p>
                        }.into_view();
                    }
                    view! {
                        <div class="overflow-x-auto">
                            <table class="w-full text-left text-xs">
                                <thead class="text-[10px] uppercase text-slate-500 border-b border-slate-100">
                                    <tr>
                                        <th class="py-2 pr-2">"Ruta"</th>
                                        <th class="py-2 pr-2 hidden sm:table-cell">"Horario"</th>
                                        <th class="py-2 pr-2">"Paradas"</th>
                                        <th class="py-2 pr-2 hidden sm:table-cell">"Camión"</th>
                                        <th class="py-2 pr-2">"Estado"</th>
                                        <th class="py-2 text-right">"Acciones"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {list.into_iter().map(|r| {
                                        let id = r.id_string();
                                        let id_toggle = id.clone();
                                        let id_delete = id.clone();
                                        let active = r.is_active;
                                        let name_delete = r.name.clone();
                                        let for_edit = r.clone();
                                        view! {
                                            <tr class="border-b border-slate-50 align-middle">
                                                <td class="py-2 pr-2">
                                                    <div class="flex items-center gap-2">
                                                        <span class="w-3 h-3 rounded-full shrink-0" style=format!("background:{}", r.color)></span>
                                                        <div>
                                                            <p class="font-semibold text-slate-900">{r.name.clone()}</p>
                                                            <p class="text-[10px] text-slate-500">
                                                                {format!("{} · {}{}", if r.district_display.is_empty() { r.district.clone() } else { r.district_display.clone() }, waste_label(&r.waste_type), if r.snap_to_roads { " · 🛣️ por calles" } else { "" })}
                                                            </p>
                                                        </div>
                                                    </div>
                                                </td>
                                                <td class="py-2 pr-2 hidden sm:table-cell text-slate-600">{if r.schedule.is_empty() { "—".to_string() } else { r.schedule.clone() }}</td>
                                                <td class="py-2 pr-2 font-mono">{r.stops.len()}</td>
                                                <td class="py-2 pr-2 hidden sm:table-cell">{r.default_truck_code.clone().unwrap_or_else(|| "—".into())}</td>
                                                <td class="py-2 pr-2">
                                                    <span class=if active { "text-[10px] font-bold px-2 py-0.5 rounded-full bg-emerald-100 text-emerald-800" } else { "text-[10px] font-bold px-2 py-0.5 rounded-full bg-slate-100 text-slate-600" }>
                                                        {if active { "Publicada" } else { "Borrador" }}
                                                    </span>
                                                </td>
                                                <td class="py-2 text-right whitespace-nowrap space-x-1">
                                                    <button class="px-2 py-1 rounded-lg font-semibold text-slate-700 hover:bg-slate-100"
                                                        on:click=move |_| editing.set(Some(Some(for_edit.clone())))>"✏️ Editar"</button>
                                                    <button class="px-2 py-1 rounded-lg font-semibold text-slate-700 hover:bg-slate-100"
                                                        on:click=move |_| api_send("PATCH", route_detail_url(&id_toggle), Some(serde_json::json!({ "is_active": !active })),
                                                            if active { "Ruta retirada del mapa ciudadano.".into() } else { "Ruta publicada.".into() })>
                                                        {if active { "Retirar" } else { "Publicar" }}
                                                    </button>
                                                    <button class="px-2 py-1 rounded-lg font-semibold text-red-600 hover:bg-red-50"
                                                        on:click=move |_| {
                                                            let ok = web_sys::window().and_then(|w| w.confirm_with_message(&format!("¿Eliminar la ruta \"{}\"? Esta acción no se puede deshacer.", name_delete)).ok()).unwrap_or(false);
                                                            if ok { api_send("DELETE", route_detail_url(&id_delete), None, "Ruta eliminada.".into()); }
                                                        }>"🗑"</button>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </div>
                    }.into_view()
                }}
            </section>

            // Camiones
            <section class=format!("{} p-4 space-y-3", card)>
                <div class="flex items-center justify-between gap-2">
                    <div>
                        <h3 class="text-sm font-bold text-slate-900">"🚚 Camiones y conductores"</h3>
                        <p class="text-[11px] text-slate-500">"Para habilitar a un conductor, agrégalo al grupo “recolectores” y asígnale un camión."</p>
                    </div>
                    <a class="px-3 py-2 rounded-xl text-xs font-semibold border border-slate-200 hover:bg-slate-50 text-slate-700"
                        href=admin_url("alerts/truck/") target="_blank" rel="noopener">
                        "Gestionar en administración ↗"
                    </a>
                </div>
                {move || {
                    let list = trucks.get();
                    if list.is_empty() {
                        return view! { <p class="text-xs text-slate-500">"No hay camiones registrados."</p> }.into_view();
                    }
                    view! {
                        <div class="grid sm:grid-cols-2 lg:grid-cols-4 gap-2">
                            {list.into_iter().map(|t| view! {
                                <div class="rounded-xl border border-slate-200 px-3 py-2">
                                    <div class="flex items-center justify-between">
                                        <p class="text-xs font-bold text-slate-900">{format!("{} · {}", t.code, t.plate)}</p>
                                        <span class=if t.is_active { "w-2 h-2 rounded-full bg-emerald-500" } else { "w-2 h-2 rounded-full bg-slate-300" }></span>
                                    </div>
                                    <p class="text-[11px] text-slate-500">
                                        {t.driver_username.clone().map(|d| format!("Conductor: {}", d)).unwrap_or_else(|| "Sin conductor asignado".into())}
                                    </p>
                                    <p class="text-[10px] text-slate-400">{waste_label(&t.waste_type)}</p>
                                </div>
                            }).collect_view()}
                        </div>
                    }.into_view()
                }}
            </section>

            // Editor modal
            {move || editing.get().map(|target| view! {
                <RouteEditor
                    route=target
                    trucks=trucks.get_untracked()
                    waste_points=waste_points
                    current_user=current_user
                    on_close=move |saved: bool| {
                        editing.set(None);
                        if saved {
                            message.set(Some((true, "Ruta guardada.".into())));
                            reload();
                        }
                    }
                />
            })}
        </div>
    }
}
