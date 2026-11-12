//! Modo Recolector: el conductor del camión inicia su turno y comparte su GPS en vivo.
//!
//! La lectura del GPS, el buffer offline y el Wake Lock viven en `window.ecoTracker`
//! (index.html); este componente orquesta el turno contra la API y muestra el estado.

use leptos::*;
use wasm_bindgen::prelude::*;

use crate::api::{driver_context_url, run_action_url, run_start_url};
use crate::models::{AuthUser, DriverContext, DriverRun, TrackerStatus};

fn api_error_message(body: &serde_json::Value, fallback: &str) -> String {
    if let Some(msg) = body.get("error").and_then(|v| v.as_str()) {
        return msg.to_string();
    }
    if let Some(msg) = body.get("detail").and_then(|v| v.as_str()) {
        return msg.to_string();
    }
    if let Some(first) = body.as_array().and_then(|a| a.first()).and_then(|v| v.as_str()) {
        return first.to_string();
    }
    fallback.to_string()
}

fn tracker_start(run_id: i64, token: &str) {
    if let Ok(tok) = serde_json::to_string(token) {
        let _ = js_sys::eval(&format!(
            "window.ecoTracker && window.ecoTracker.start({}, {});",
            run_id, tok
        ));
    }
}

fn tracker_stop(clear_queue: bool) {
    let _ = js_sys::eval(&format!(
        "window.ecoTracker && window.ecoTracker.stop({});",
        clear_queue
    ));
}

fn tracker_flush() {
    let _ = js_sys::eval("window.ecoTracker && window.ecoTracker.flush();");
}

fn ago_label(ts_ms: Option<f64>, now_ms: f64) -> String {
    match ts_ms {
        None => "—".to_string(),
        Some(ts) => {
            let s = ((now_ms - ts) / 1000.0).max(0.0).round() as i64;
            if s < 60 {
                format!("hace {} s", s)
            } else {
                format!("hace {} min", s / 60)
            }
        }
    }
}

fn gps_label(gps: &str) -> (&'static str, &'static str) {
    match gps {
        "ok" => ("🟢 Señal GPS buena", "text-emerald-700"),
        "weak" => ("🟡 Señal GPS débil", "text-amber-700"),
        "searching" => ("🔎 Buscando satélites…", "text-slate-600"),
        "denied" => ("⛔ Permiso de ubicación denegado", "text-red-700"),
        "unavailable" => ("⚠️ GPS no disponible", "text-red-700"),
        _ => ("⚪ GPS inactivo", "text-slate-500"),
    }
}

#[component]
pub fn CollectorPanel(
    current_user: RwSignal<Option<AuthUser>>,
    show_panel: RwSignal<bool>,
) -> impl IntoView {
    let context = create_rw_signal(None::<DriverContext>);
    let loading = create_rw_signal(false);
    let busy = create_rw_signal(false);
    let error_msg = create_rw_signal(None::<String>);
    let selected_truck = create_rw_signal(None::<i64>);
    let selected_route = create_rw_signal(None::<i64>);
    let tracker = create_rw_signal(TrackerStatus::default());
    let now_ms = create_rw_signal(js_sys::Date::now());

    let is_collector = move || current_user.with(|u| u.as_ref().map(|u| u.is_collector).unwrap_or(false));
    let token = move || current_user.with_untracked(|u| u.as_ref().map(|u| u.token.clone()));
    let open_run = move || context.with(|c| c.as_ref().and_then(|c| c.open_run.clone()));

    // Puente JS → Rust: cada cambio del rastreador actualiza la señal `tracker`
    {
        let on_status = Closure::<dyn FnMut(String)>::new(move |raw: String| {
            if let Ok(st) = serde_json::from_str::<TrackerStatus>(&raw) {
                if tracker.get_untracked() != st {
                    tracker.set(st);
                }
            }
        });
        if let Some(win) = web_sys::window() {
            let _ = js_sys::Reflect::set(&win, &JsValue::from_str("onTrackerStatus"), on_status.as_ref().unchecked_ref());
        }
        on_status.forget();
    }

    // Reloj para las etiquetas "hace N s"
    let _ = set_interval_with_handle(move || now_ms.set(js_sys::Date::now()), std::time::Duration::from_secs(1));

    let load_context = move || {
        let Some(tok) = token() else { return };
        loading.set(true);
        spawn_local(async move {
            let resp = gloo_net::http::Request::get(&driver_context_url())
                .header("Authorization", &format!("Token {}", tok))
                .send()
                .await;
            loading.set(false);
            match resp {
                Ok(r) if r.ok() => match r.json::<DriverContext>().await {
                    Ok(ctx) => {
                        // Preselección cómoda: único camión y su ruta habitual
                        if selected_truck.get_untracked().is_none() && ctx.trucks.len() == 1 {
                            let t = &ctx.trucks[0];
                            selected_truck.set(Some(t.id));
                            if let Some(route) = ctx.routes.iter().find(|r| r.default_truck_code.as_deref() == Some(t.code.as_str())) {
                                selected_route.set(route.id.as_i64());
                            }
                        }
                        if selected_route.get_untracked().is_none() && ctx.routes.len() == 1 {
                            selected_route.set(ctx.routes[0].id.as_i64());
                        }
                        // Reanudar el GPS tras recargar la página si el turno sigue en recorrido
                        match &ctx.open_run {
                            Some(run) if run.status == "active" => tracker_start(run.id, &tok),
                            Some(_) => tracker_stop(false),
                            None => tracker_stop(true),
                        }
                        error_msg.set(None);
                        context.set(Some(ctx));
                    }
                    Err(_) => error_msg.set(Some("Respuesta inesperada del servidor.".into())),
                },
                Ok(r) if r.status() == 403 => {
                    error_msg.set(Some("Tu cuenta no está habilitada como conductor recolector.".into()));
                }
                Ok(r) => error_msg.set(Some(format!("No se pudo cargar tu turno (HTTP {}).", r.status()))),
                Err(_) => error_msg.set(Some("Sin conexión con el servidor.".into())),
            }
        });
    };

    // Al iniciar sesión como conductor: cargar contexto (y reanudar GPS si había turno activo)
    create_effect(move |prev: Option<bool>| {
        let collector = is_collector();
        if collector && prev != Some(true) {
            load_context();
        }
        if !collector && prev == Some(true) {
            tracker_stop(false);
            context.set(None);
            show_panel.set(false);
        }
        collector
    });

    // El servidor cerró o pausó el turno (409 desde el rastreador): refrescar
    create_effect(move |_| {
        let st = tracker.with(|t| t.server_status.clone());
        if matches!(st.as_deref(), Some("finished") | Some("paused")) && !tracker.with_untracked(|t| t.tracking) {
            load_context();
        }
    });

    let post_run = move |url: String, body: Option<serde_json::Value>, after: Box<dyn Fn(DriverRun)>| {
        let Some(tok) = token() else { return };
        busy.set(true);
        error_msg.set(None);
        spawn_local(async move {
            let builder = gloo_net::http::Request::post(&url).header("Authorization", &format!("Token {}", tok));
            let resp = match body {
                Some(b) => match builder.json(&b) {
                    Ok(req) => req.send().await,
                    Err(e) => Err(e),
                },
                None => builder.send().await,
            };
            busy.set(false);
            match resp {
                Ok(r) if r.ok() => {
                    if let Ok(run) = r.json::<DriverRun>().await {
                        after(run);
                    }
                }
                Ok(r) => {
                    let body = r.json::<serde_json::Value>().await.unwrap_or_default();
                    error_msg.set(Some(api_error_message(&body, &format!("Error HTTP {}", r.status()))));
                    load_context();
                }
                Err(_) => error_msg.set(Some("Sin conexión. Intenta de nuevo en unos segundos.".into())),
            }
        });
    };

    let set_open_run = move |run: Option<DriverRun>| {
        context.update(|c| {
            if let Some(c) = c {
                c.open_run = run;
            }
        });
    };

    let on_start = move |_| {
        let (Some(truck), Some(route)) = (selected_truck.get_untracked(), selected_route.get_untracked()) else {
            error_msg.set(Some("Elige tu camión y la ruta de hoy.".into()));
            return;
        };
        let tok = token().unwrap_or_default();
        post_run(
            run_start_url(),
            Some(serde_json::json!({ "truck": truck, "route": route })),
            Box::new(move |run| {
                tracker_start(run.id, &tok);
                set_open_run(Some(run));
            }),
        );
    };

    let on_pause = move |_| {
        let Some(run) = open_run() else { return };
        tracker_flush();
        post_run(
            run_action_url(run.id, "pause"),
            None,
            Box::new(move |run| {
                tracker_stop(false);
                set_open_run(Some(run));
            }),
        );
    };

    let on_resume = move |_| {
        let Some(run) = open_run() else { return };
        let tok = token().unwrap_or_default();
        post_run(
            run_action_url(run.id, "resume"),
            None,
            Box::new(move |run| {
                tracker_start(run.id, &tok);
                set_open_run(Some(run));
            }),
        );
    };

    let on_finish = move |_| {
        let Some(run) = open_run() else { return };
        let confirmed = web_sys::window()
            .and_then(|w| w.confirm_with_message("¿Finalizar el recorrido? El camión dejará de mostrarse en el mapa.").ok())
            .unwrap_or(false);
        if !confirmed {
            return;
        }
        tracker_flush();
        post_run(
            run_action_url(run.id, "finish"),
            None,
            Box::new(move |_run| {
                tracker_stop(true);
                set_open_run(None);
            }),
        );
    };

    let select_class = "w-full rounded-xl border border-slate-200 bg-white px-3 py-2.5 text-sm text-slate-800 focus:outline-none focus:ring-2 focus:ring-emerald-500";

    view! {
        // Píldora flotante mientras el panel está cerrado y hay un turno abierto
        {move || (is_collector() && !show_panel.get() && open_run().is_some()).then(|| {
            let st = tracker.get();
            let paused = open_run().map(|r| r.status == "paused").unwrap_or(false);
            let (label, dot) = if paused {
                ("En pausa", "bg-amber-400")
            } else if st.tracking && (st.gps == "ok" || st.gps == "weak") {
                ("Compartiendo GPS", "bg-emerald-400 animate-pulse")
            } else {
                ("GPS sin señal", "bg-red-500 animate-pulse")
            };
            view! {
                <button
                    class="fixed bottom-24 left-3 md:bottom-6 md:left-auto md:right-6 z-40 bg-slate-900/95 text-white rounded-full pl-3 pr-4 py-2 shadow-xl flex items-center space-x-2 text-xs font-semibold cursor-pointer"
                    on:click=move |_| show_panel.set(true)
                >
                    <span class=format!("w-2.5 h-2.5 rounded-full {}", dot)></span>
                    <span>"🚛 " {label}</span>
                </button>
            }
        })}

        {move || (show_panel.get() && is_collector()).then(|| view! {
            <div class="fixed inset-0 bg-slate-900/50 backdrop-blur-sm flex items-end sm:items-center justify-center p-0 sm:p-4 z-50">
                <div class="bg-white w-full sm:max-w-md rounded-t-3xl sm:rounded-2xl border border-slate-200/90 p-5 shadow-[0_24px_64px_rgba(0,0,0,0.18)] max-h-[90vh] overflow-y-auto space-y-4">
                    <div class="w-12 h-1.5 bg-slate-300 rounded-full mx-auto -mt-1 mb-1 sm:hidden"></div>

                    <div class="flex items-center justify-between pb-3 border-b border-slate-100">
                        <div class="flex items-center space-x-2.5">
                            <div class="w-9 h-9 rounded-xl bg-emerald-50 border border-emerald-100 flex items-center justify-center text-lg">"🚛"</div>
                            <div>
                                <h3 class="text-sm font-bold text-slate-900 leading-tight">"Modo Recolector"</h3>
                                <p class="text-[11px] text-slate-500">"Comparte la ubicación del camión con los vecinos"</p>
                            </div>
                        </div>
                        <button
                            class="text-slate-400 hover:text-slate-600 p-1.5 rounded-lg transition cursor-pointer"
                            title="Cerrar (el GPS sigue activo)"
                            on:click=move |_| show_panel.set(false)
                        >
                            <i class="fa-solid fa-xmark"></i>
                        </button>
                    </div>

                    {move || error_msg.get().map(|m| view! {
                        <div class="rounded-xl bg-red-50 border border-red-200 text-red-700 text-xs px-3 py-2">{m}</div>
                    })}

                    {move || {
                        if loading.get() && context.with(|c| c.is_none()) {
                            return view! { <p class="text-xs text-slate-500 py-6 text-center">"Cargando tu camión y rutas…"</p> }.into_view();
                        }
                        let Some(ctx) = context.get() else {
                            return view! {
                                <button class="w-full rounded-xl border border-slate-200 py-2.5 text-sm font-semibold text-slate-700 cursor-pointer" on:click=move |_| load_context()>
                                    "Reintentar"
                                </button>
                            }.into_view();
                        };

                        match ctx.open_run.clone() {
                            None => {
                                if ctx.trucks.is_empty() {
                                    return view! {
                                        <p class="text-xs text-slate-600 bg-amber-50 border border-amber-200 rounded-xl px-3 py-2">
                                            "No tienes un camión asignado. Pide a la autoridad municipal que te asigne uno en el panel de administración."
                                        </p>
                                    }.into_view();
                                }
                                let trucks = ctx.trucks.clone();
                                let routes = ctx.routes.clone();
                                view! {
                                    <div class="space-y-3">
                                        <label class="block space-y-1">
                                            <span class="text-[11px] font-semibold text-slate-600 uppercase tracking-wide">"Camión"</span>
                                            <select
                                                class=select_class
                                                on:change=move |ev| selected_truck.set(event_target_value(&ev).parse::<i64>().ok())
                                            >
                                                <option value="" selected=move || selected_truck.get().is_none()>"Selecciona tu camión"</option>
                                                {trucks.into_iter().map(|t| {
                                                    let id = t.id;
                                                    view! {
                                                        <option value=id.to_string() selected=move || selected_truck.get() == Some(id)>
                                                            {format!("{} · {}", t.code, t.plate)}
                                                        </option>
                                                    }
                                                }).collect_view()}
                                            </select>
                                        </label>
                                        <label class="block space-y-1">
                                            <span class="text-[11px] font-semibold text-slate-600 uppercase tracking-wide">"Ruta de hoy"</span>
                                            <select
                                                class=select_class
                                                on:change=move |ev| selected_route.set(event_target_value(&ev).parse::<i64>().ok())
                                            >
                                                <option value="" selected=move || selected_route.get().is_none()>"Selecciona la ruta"</option>
                                                {routes.into_iter().filter_map(|r| {
                                                    let id = r.id.as_i64()?;
                                                    Some(view! {
                                                        <option value=id.to_string() selected=move || selected_route.get() == Some(id)>
                                                            {if r.schedule.is_empty() { r.name.clone() } else { format!("{} ({})", r.name, r.schedule) }}
                                                        </option>
                                                    })
                                                }).collect_view()}
                                            </select>
                                        </label>
                                        <div class="text-[11px] text-slate-600 bg-slate-50 border border-slate-200 rounded-xl px-3 py-2 space-y-1">
                                            <p>"📱 Coloca el teléfono en el soporte del camión, conectado al cargador."</p>
                                            <p>"🔆 Mantén EcoAlerta abierta y la pantalla encendida: si la app pasa a segundo plano, el GPS se detiene."</p>
                                        </div>
                                        <button
                                            class="w-full rounded-xl bg-emerald-600 hover:bg-emerald-700 disabled:opacity-60 text-white py-3 text-sm font-bold shadow cursor-pointer"
                                            disabled=move || busy.get()
                                            on:click=on_start
                                        >
                                            {move || if busy.get() { "Iniciando…" } else { "▶ Iniciar recorrido" }}
                                        </button>
                                    </div>
                                }.into_view()
                            }
                            Some(run) => {
                                let paused = run.status == "paused";
                                view! {
                                    <div class="space-y-3">
                                        <div class="rounded-2xl border border-slate-200 p-3 flex items-center justify-between">
                                            <div>
                                                <p class="text-sm font-bold text-slate-900">{run.route_name.clone()}</p>
                                                <p class="text-[11px] text-slate-500">{format!("Camión {}", run.truck_code)}</p>
                                            </div>
                                            <span class=if paused {
                                                "text-[11px] font-bold px-2 py-1 rounded-full bg-amber-100 text-amber-800"
                                            } else {
                                                "text-[11px] font-bold px-2 py-1 rounded-full bg-emerald-100 text-emerald-800"
                                            }>
                                                {if paused { "⏸ En pausa" } else { "● En recorrido" }}
                                            </span>
                                        </div>

                                        {(!paused).then(|| view! {
                                            <div class="rounded-2xl bg-slate-50 border border-slate-200 p-3 space-y-1.5 text-xs">
                                                {move || {
                                                    let st = tracker.get();
                                                    let (label, cls) = gps_label(&st.gps);
                                                    let now = now_ms.get();
                                                    view! {
                                                        <p class=format!("font-semibold {}", cls)>{label}</p>
                                                        <div class="grid grid-cols-2 gap-x-3 gap-y-1 text-slate-600">
                                                            <span>"Precisión"</span>
                                                            <span class="text-right font-medium text-slate-800">
                                                                {st.accuracy.map(|a| format!("± {:.0} m", a)).unwrap_or_else(|| "—".into())}
                                                            </span>
                                                            <span>"Última lectura"</span>
                                                            <span class="text-right font-medium text-slate-800">{ago_label(st.last_fix_at, now)}</span>
                                                            <span>"Último envío"</span>
                                                            <span class="text-right font-medium text-slate-800">{ago_label(st.last_sent_at, now)}</span>
                                                            <span>"Pendientes por enviar"</span>
                                                            <span class="text-right font-medium text-slate-800">{st.pending}</span>
                                                            <span>"Pantalla siempre encendida"</span>
                                                            <span class="text-right font-medium text-slate-800">{if st.wake_lock { "Sí" } else { "No disponible" }}</span>
                                                        </div>
                                                        {st.error.clone().map(|e| view! { <p class="text-[11px] text-amber-700 pt-1">{e}</p> })}
                                                    }
                                                }}
                                            </div>
                                        })}

                                        <div class="grid grid-cols-2 gap-2">
                                            {if paused {
                                                view! {
                                                    <button class="rounded-xl bg-emerald-600 hover:bg-emerald-700 disabled:opacity-60 text-white py-2.5 text-sm font-bold cursor-pointer"
                                                        disabled=move || busy.get() on:click=on_resume>
                                                        "▶ Reanudar"
                                                    </button>
                                                }.into_view()
                                            } else {
                                                view! {
                                                    <button class="rounded-xl bg-amber-500 hover:bg-amber-600 disabled:opacity-60 text-white py-2.5 text-sm font-bold cursor-pointer"
                                                        disabled=move || busy.get() on:click=on_pause>
                                                        "⏸ Pausar"
                                                    </button>
                                                }.into_view()
                                            }}
                                            <button class="rounded-xl bg-slate-900 hover:bg-black disabled:opacity-60 text-white py-2.5 text-sm font-bold cursor-pointer"
                                                disabled=move || busy.get() on:click=on_finish>
                                                "■ Finalizar"
                                            </button>
                                        </div>
                                        <p class="text-[10px] text-slate-500 text-center">
                                            "Puedes cerrar este panel: el GPS seguirá enviándose mientras la app esté abierta."
                                        </p>
                                    </div>
                                }.into_view()
                            }
                        }
                    }}
                </div>
            </div>
        })}
    }
}
