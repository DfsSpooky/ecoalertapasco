use leptos::*;
use wasm_bindgen::prelude::*;
use crate::components::alerts_carousel::AlertsCarousel;
use crate::geo::get_browser_gps_position;
use crate::models::{AuthUser, EcoAlert, TRUCK_ROUTE_POINTS, WastePoint};

#[component]
pub fn MapView(
    filtered_alerts: Memo<Vec<EcoAlert>>,
    selected_alert_id: RwSignal<Option<String>>,
    is_report_mode: RwSignal<bool>,
    report_center: RwSignal<(f64, f64)>,
    show_new_report_dialog: RwSignal<bool>,
    waste_points: RwSignal<Vec<WastePoint>>,
    show_waste_layer: RwSignal<bool>,
    selected_waste_point_id: RwSignal<Option<String>>,
    show_waste_list_modal: RwSignal<bool>,
    show_transparency_portal: RwSignal<bool>,
    show_authority_dashboard: RwSignal<bool>,
    current_user: RwSignal<Option<AuthUser>>,
    show_help_guide: RwSignal<bool>,
    is_sidebar_collapsed: RwSignal<bool>,
) -> impl IntoView {
    let map_layer = create_rw_signal("osm".to_string());
    let is_locating = create_rw_signal(false);
    let show_heatmap = create_rw_signal(false);
    let show_optimized_route = create_rw_signal(false);

    create_effect(move |_| {
        let on_alert_selected = Closure::wrap(Box::new(move |id: String| {
            selected_alert_id.set(Some(id));
        }) as Box<dyn FnMut(String)>);
        js_sys::Reflect::set(
            &web_sys::window().unwrap(),
            &JsValue::from_str("onAlertSelected"),
            on_alert_selected.as_ref().unchecked_ref(),
        ).unwrap();
        on_alert_selected.forget();

        let on_waste_selected = Closure::wrap(Box::new(move |id: String| {
            selected_waste_point_id.set(Some(id));
        }) as Box<dyn FnMut(String)>);
        js_sys::Reflect::set(
            &web_sys::window().unwrap(),
            &JsValue::from_str("onWastePointSelected"),
            on_waste_selected.as_ref().unchecked_ref(),
        ).unwrap();
        on_waste_selected.forget();

        let on_center_changed = Closure::wrap(Box::new(move |lat: f64, lng: f64| {
            report_center.set((lat, lng));
        }) as Box<dyn FnMut(f64, f64)>);
        js_sys::Reflect::set(
            &web_sys::window().unwrap(),
            &JsValue::from_str("onMapCenterChanged"),
            on_center_changed.as_ref().unchecked_ref(),
        ).unwrap();
        on_center_changed.forget();

        let _ = js_sys::eval("window.ecoMap.init('map-container', -10.6675, -76.2567, 14);");
    });

    create_effect(move |_| {
        let alerts = filtered_alerts.get();
        if let Ok(json_str) = serde_json::to_string(&alerts) {
            let script = format!("window.ecoMap.renderMarkers({});", json_str);
            let _ = js_sys::eval(&script);
        }
    });

    create_effect(move |_| {
        let points = waste_points.get();
        let visible = show_waste_layer.get();
        if let Ok(json_str) = serde_json::to_string(&points) {
            let script = format!("window.ecoMap.renderWastePoints({}, {});", json_str, visible);
            let _ = js_sys::eval(&script);
        }
        if let Ok(route_json) = serde_json::to_string(&TRUCK_ROUTE_POINTS) {
            let script = format!("window.ecoMap.renderTruckRoute({}, {});", route_json, visible);
            let _ = js_sys::eval(&script);
        }
    });

    create_effect(move |_| {
        let alerts = filtered_alerts.get();
        let is_heat = show_heatmap.get();
        if let Ok(json_str) = serde_json::to_string(&alerts) {
            let script = format!("window.ecoMap.renderHeatmap({}, {});", json_str, is_heat);
            let _ = js_sys::eval(&script);
        }
    });

    create_effect(move |_| {
        let points = waste_points.get();
        let is_opt = show_optimized_route.get();
        if let Ok(json_str) = serde_json::to_string(&points) {
            let script = format!("window.ecoMap.renderOptimizedRoute({}, {});", json_str, is_opt);
            let _ = js_sys::eval(&script);
        }
    });

    // Mantener Leaflet sincronizado ante colapso/expansión del sidebar en desktop
    create_effect(move |_| {
        let _ = is_sidebar_collapsed.get();
        let _ = js_sys::eval(r#"
            if (window.ecoMap && window.ecoMap.map) {
                window.ecoMap.map.invalidateSize({ animate: false });
                setTimeout(() => {
                    if (window.ecoMap && window.ecoMap.map) {
                        window.ecoMap.map.invalidateSize({ animate: false });
                    }
                }, 320);
            }
        "#);
    });

    // Notificar modo reporte a Leaflet para permitir clics directos sobre el mapa
    create_effect(move |_| {
        let is_rep = is_report_mode.get();
        let _ = js_sys::eval(&format!(r#"
            window.isReportMode = {};
            if (window.ecoMap && window.ecoMap.map) {{
                window.ecoMap.map.invalidateSize({{ animate: false }});
            }}
        "#, is_rep));
    });

    let switch_layer = move |style: &'static str| {
        map_layer.set(style.to_string());
        let _ = js_sys::eval(&format!("window.ecoMap.setLayer('{}');", style));
    };

    let locate_user = move |_| {
        is_locating.set(true);
        spawn_local(async move {
            let _ = js_sys::eval("if (window.ecoMap && window.ecoMap.map) { window.ecoMap.map.invalidateSize({ animate: false }); }");
            if let Ok((lat, lng)) = get_browser_gps_position().await {
                report_center.set((lat, lng));
                let _ = js_sys::eval(&format!("window.ecoMap.setUserLocation({}, {});", lat, lng));
            }
            is_locating.set(false);
        });
    };

    view! {
        <main class="flex-1 h-full relative overflow-hidden bg-slate-100">
            // Full bleed map container
            <div id="map-container" class="w-full h-full z-0"></div>

            // Mobile Hamburger Menu Button (opens sidebar drawer on mobile)
            <div class="absolute top-2.5 left-2.5 z-10 md:hidden">
                <button
                    class="bg-white/95 backdrop-blur-md p-2 rounded-xl shadow-[0_2px_8px_rgba(60,64,67,0.15)] border border-slate-200/90 text-slate-800 hover:text-emerald-700 active:scale-95 transition flex items-center justify-center cursor-pointer"
                    on:click=move |_| is_sidebar_collapsed.set(false)
                    title="Abrir menú y filtros"
                >
                    <i class="fa-solid fa-bars text-xs"></i>
                </button>
            </div>

            // Google Maps Style Floating Island: Layer Switcher, Waste Points, & GPS
            <div class="absolute top-2.5 left-12 right-2.5 md:left-auto md:right-3.5 z-10 flex items-center justify-start md:justify-end gap-1.5 overflow-x-auto md:overflow-visible no-scrollbar py-0.5 scroll-smooth touch-pan-x">
                // Toggle Waste & Trucks Layer Button
                <button
                    class=move || {
                        if show_waste_layer.get() {
                            "shrink-0 whitespace-nowrap bg-emerald-600 hover:bg-emerald-700 text-white shadow-[0_2px_8px_rgba(16,185,129,0.3)] border border-emerald-500/80 px-3 py-1.5 rounded-full text-xs font-semibold flex items-center space-x-1.5 transition cursor-pointer"
                        } else {
                            "shrink-0 whitespace-nowrap bg-white/95 hover:bg-white text-slate-700 hover:text-slate-900 shadow-[0_2px_8px_rgba(60,64,67,0.12)] border border-slate-200/90 px-3 py-1.5 rounded-full text-xs font-medium flex items-center space-x-1.5 transition cursor-pointer"
                        }
                    }
                    title="Mostrar botaderos autorizados, puntos de acopio y ruta del camión recolector"
                    on:click=move |_| show_waste_layer.update(|v| *v = !*v)
                >
                    <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M7 19H4.815a1.83 1.83 0 0 1-1.57-.881 1.785 1.785 0 0 1-.004-1.784L7.196 9.5"/>
                        <path d="M11 19h8.2a1.8 1.8 0 0 0 1.573-.896 1.79 1.79 0 0 0 0-1.792L16.8 9.5"/>
                        <path d="m15.5 13-3.5 6-3.5-6"/>
                        <path d="m2 9 3-5 3 5"/>
                    </svg>
                    <span>"♻️ Acopio"</span>
                </button>

                // Toggle Heatmap Button
                <button
                    class=move || {
                        if show_heatmap.get() {
                            "shrink-0 whitespace-nowrap bg-orange-600 hover:bg-orange-700 text-white shadow-[0_2px_8px_rgba(234,88,12,0.3)] border border-orange-500/80 px-3 py-1.5 rounded-full text-xs font-semibold flex items-center space-x-1.5 transition cursor-pointer"
                        } else {
                            "shrink-0 whitespace-nowrap bg-white/95 hover:bg-white text-slate-700 hover:text-slate-900 shadow-[0_2px_8px_rgba(60,64,67,0.12)] border border-slate-200/90 px-3 py-1.5 rounded-full text-xs font-medium flex items-center space-x-1.5 transition cursor-pointer"
                        }
                    }
                    title="Alternar entre marcadores normales y Mapa de Calor por severidad"
                    on:click=move |_| show_heatmap.update(|v| *v = !*v)
                >
                    <span>"🔥"</span>
                    <span>"Calor"</span>
                </button>

                // Toggle Optimized Collection Route
                <button
                    class=move || {
                        if show_optimized_route.get() {
                            "shrink-0 whitespace-nowrap bg-teal-600 hover:bg-teal-700 text-white shadow-[0_2px_8px_rgba(13,148,136,0.3)] border border-teal-500/80 px-3 py-1.5 rounded-full text-xs font-semibold flex items-center space-x-1.5 transition cursor-pointer"
                        } else {
                            "shrink-0 whitespace-nowrap bg-white/95 hover:bg-white text-slate-700 hover:text-slate-900 shadow-[0_2px_8px_rgba(60,64,67,0.12)] border border-slate-200/90 px-3 py-1.5 rounded-full text-xs font-medium flex items-center space-x-1.5 transition cursor-pointer"
                        }
                    }
                    title="Calcular y trazar la ruta de recolección prioritaria para contenedores llenos y medios"
                    on:click=move |_| show_optimized_route.update(|v| *v = !*v)
                >
                    <span>"🚛"</span>
                    <span>"Ruta Óptima"</span>
                </button>

                // Nearest Waste Points List Button
                <button
                    class="shrink-0 whitespace-nowrap bg-white/95 hover:bg-white text-slate-700 hover:text-slate-900 shadow-[0_2px_8px_rgba(60,64,67,0.12)] border border-slate-200/90 px-3 py-1.5 rounded-full text-xs font-medium flex items-center space-x-1.5 transition cursor-pointer"
                    title="Listar puntos de acopio cercanos ordenados por proximidad"
                    on:click=move |_| show_waste_list_modal.set(true)
                >
                    <svg class="w-3.5 h-3.5 text-slate-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <line x1="8" y1="6" x2="21" y2="6"></line>
                        <line x1="8" y1="12" x2="21" y2="12"></line>
                        <line x1="8" y1="18" x2="21" y2="18"></line>
                        <line x1="3" y1="6" x2="3.01" y2="6"></line>
                        <line x1="3" y1="12" x2="3.01" y2="12"></line>
                        <line x1="3" y1="18" x2="3.01" y2="18"></line>
                    </svg>
                    <span>"Puntos Cercanos"</span>
                </button>

                // Transparency Portal Quick Button
                <button
                    class="shrink-0 whitespace-nowrap bg-white/95 hover:bg-white text-emerald-800 hover:text-emerald-900 shadow-[0_2px_8px_rgba(60,64,67,0.12)] border border-emerald-200/80 px-3 py-1.5 rounded-full text-xs font-semibold flex items-center space-x-1.5 transition cursor-pointer"
                    title="Ver Portal de Transparencia Ciudadana y Casos Resueltos"
                    on:click=move |_| show_transparency_portal.set(true)
                >
                    <span>"🏆"</span>
                    <span>"Transparencia"</span>
                </button>

                // Authority Dashboard Quick Button (Restricted exclusively to authorized staff)
                {move || {
                    let is_authority = current_user.get().map(|u| u.is_authority()).unwrap_or(false);
                    is_authority.then(|| view! {
                        <button
                            class="shrink-0 whitespace-nowrap bg-white/95 hover:bg-white text-slate-800 hover:text-slate-900 shadow-[0_2px_8px_rgba(60,64,67,0.12)] border border-slate-200/90 px-3 py-1.5 rounded-full text-xs font-semibold flex items-center space-x-1.5 transition cursor-pointer"
                            title="Panel de Control de Autoridades (Acceso Fiscalizador)"
                            on:click=move |_| show_authority_dashboard.set(true)
                        >
                            <span>"🏛️"</span>
                            <span>"Autoridades"</span>
                        </button>
                    })
                }}
                // Google Help Center Quick Button
                <button
                    class="shrink-0 whitespace-nowrap bg-white/95 hover:bg-white text-slate-700 hover:text-blue-600 shadow-[0_1px_3px_rgba(60,64,67,0.15)] border border-slate-200/90 px-3 py-1.5 rounded-full text-xs font-medium flex items-center space-x-1.5 transition cursor-pointer group"
                    title="Centro de Ayuda · Guía Ciudadana y SLA"
                    on:click=move |_| show_help_guide.set(true)
                >
                    <i class="fa-solid fa-circle-question text-blue-600 group-hover:scale-110 transition-transform text-xs"></i>
                    <span class="font-medium text-slate-800 group-hover:text-blue-600">"Ayuda"</span>
                </button>

                // GPS Center Button
                <button
                    class="shrink-0 whitespace-nowrap bg-white/95 backdrop-blur-md px-3 py-1.5 rounded-full shadow-[0_2px_8px_rgba(60,64,67,0.12)] border border-slate-200/90 text-slate-700 hover:text-slate-900 transition flex items-center space-x-1.5 hover:scale-105 cursor-pointer text-xs font-medium"
                    title="Centrar en mi ubicación GPS"
                    disabled=move || is_locating.get()
                    on:click=locate_user
                >
                    {move || if is_locating.get() {
                        view! {
                            <span class="inline-block w-3.5 h-3.5 border-2 border-slate-400 border-t-slate-800 rounded-full animate-spin"></span>
                        }.into_view()
                    } else {
                        view! {
                            <svg class="w-3.5 h-3.5 text-slate-700" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <circle cx="12" cy="12" r="3"/>
                                <path d="M12 2v3m0 14v3M2 12h3m14 0h3"/>
                            </svg>
                        }.into_view()
                    }}
                    <span>"GPS"</span>
                </button>

                // Map Style Segmented Control (Google Maps style)
                <div class="shrink-0 whitespace-nowrap bg-white/95 backdrop-blur-md rounded-full shadow-[0_2px_8px_rgba(60,64,67,0.12)] border border-slate-200/90 p-0.5 flex space-x-0.5">
                    <button
                        class=move || {
                            if map_layer.get() == "osm" {
                                "px-2.5 py-1 text-xs font-semibold rounded-full bg-slate-900 text-white shadow-2xs transition cursor-pointer"
                            } else {
                                "px-2.5 py-1 text-xs font-medium text-slate-600 hover:text-slate-900 transition cursor-pointer"
                            }
                        }
                        on:click=move |_| switch_layer("osm")
                    >
                        "Calles"
                    </button>
                    <button
                        class=move || {
                            if map_layer.get() == "satellite" {
                                "px-2.5 py-1 text-xs font-semibold rounded-full bg-slate-900 text-white shadow-2xs transition cursor-pointer"
                            } else {
                                "px-2.5 py-1 text-xs font-medium text-slate-600 hover:text-slate-900 transition cursor-pointer"
                            }
                        }
                        on:click=move |_| switch_layer("satellite")
                    >
                        "Satélite"
                    </button>
                    <button
                        class=move || {
                            if map_layer.get() == "topo" {
                                "px-2.5 py-1 text-xs font-semibold rounded-full bg-slate-900 text-white shadow-2xs transition cursor-pointer"
                            } else {
                                "px-2.5 py-1 text-xs font-medium text-slate-600 hover:text-slate-900 transition cursor-pointer"
                            }
                        }
                        on:click=move |_| switch_layer("topo")
                    >
                        "Relieve"
                    </button>
                </div>
            </div>

            // Report Mode Center Target (Flat Apple Maps Purple Drop Pin - Distinct from Red Critical Alerts)
            {move || is_report_mode.get().then(|| {
                let (lat, lng) = report_center.get();
                view! {
                    <div class="pointer-events-none absolute inset-0 flex items-center justify-center z-10">
                        // Top Floating Helper Banner (Flat Apple iOS Status Pill)
                        <div class="pointer-events-auto absolute top-3 sm:top-4 left-1/2 transform -translate-x-1/2 z-30 bg-slate-900/90 backdrop-blur-xl text-white py-1.5 px-3.5 rounded-full shadow-md border border-white/10 flex items-center justify-between space-x-3 text-xs font-medium animate-in fade-in duration-150 w-11/12 max-w-sm">
                            <div class="flex items-center space-x-2 truncate">
                                <i class="fa-solid fa-map-pin text-purple-400 text-xs shrink-0"></i>
                                <span class="truncate text-[11px] sm:text-xs text-slate-200">"Mueve el mapa al foco contaminante"</span>
                            </div>
                            <button
                                type="button"
                                class="px-2.5 py-0.5 rounded-full bg-white/15 hover:bg-white/25 active:bg-white/30 text-white text-[11px] font-semibold transition shrink-0 cursor-pointer flex items-center space-x-1"
                                on:click=move |_| is_report_mode.set(false)
                                title="Cancelar modo ubicación"
                            >
                                <i class="fa-solid fa-xmark text-[10px]"></i>
                                <span>"Salir"</span>
                            </button>
                        </div>

                        // Center Pin Target (Flat Apple Maps Purple Drop Pin)
                        <div class="relative flex flex-col items-center transform -translate-y-1/2 select-none pointer-events-none">
                            // Pin Head
                            <div class="w-10 h-10 rounded-full bg-purple-600 text-white flex items-center justify-center shadow-[0_6px_16px_rgba(147,51,234,0.35)] border-2 border-white">
                                <svg class="w-5 h-5 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4">
                                    <path d="M20 10c0 6-8 12-8 12s-8-6-8-12a8 8 0 0 1 16 0Z"/>
                                    <circle cx="12" cy="10" r="3"/>
                                </svg>
                            </div>
                            // Stem
                            <div class="w-1 h-3 bg-purple-600"></div>
                            // Subtle contact shadow
                            <div class="w-3.5 h-1.5 rounded-full bg-black/25 blur-[1px]"></div>
                        </div>

                        // Floating Confirmation Card (Flat Apple Design System)
                        <div class="pointer-events-auto absolute bottom-20 sm:bottom-6 left-1/2 transform -translate-x-1/2 bg-white/95 backdrop-blur-2xl p-3.5 sm:p-4 rounded-2xl shadow-[0_8px_30px_rgba(0,0,0,0.08)] border border-slate-200/90 flex items-center space-x-3 max-w-md w-11/12 transition z-20">
                            <div class="w-10 h-10 rounded-xl bg-purple-50 text-purple-600 border border-purple-100 flex items-center justify-center shrink-0">
                                <i class="fa-solid fa-location-dot text-base"></i>
                            </div>
                            <div class="flex-1 min-w-0">
                                <p class="text-[10px] font-bold text-slate-400 uppercase tracking-wider">
                                    "Paso 1 · Ubicación del Incidente"
                                </p>
                                <p class="text-xs text-slate-900 font-bold truncate font-mono mt-0.5">
                                    {format!("{:.4}, {:.4}", lat, lng)}
                                </p>
                            </div>
                            <button
                                type="button"
                                class="py-2 px-3 bg-slate-100 hover:bg-slate-200 active:bg-slate-300/80 text-slate-700 font-semibold text-xs rounded-xl transition shrink-0 flex items-center space-x-1.5 cursor-pointer active:scale-95"
                                on:click=locate_user
                                title="Centrar en mi GPS actual"
                            >
                                <i class="fa-solid fa-crosshairs text-slate-500 text-xs"></i>
                                <span>"Mi GPS"</span>
                            </button>
                            <button
                                type="button"
                                class="py-2 px-4 bg-slate-900 hover:bg-black active:scale-95 text-white font-semibold text-xs rounded-xl shadow-xs transition shrink-0 flex items-center space-x-1.5 cursor-pointer"
                                on:click=move |_| show_new_report_dialog.set(true)
                            >
                                <span>"Confirmar"</span>
                                <i class="fa-solid fa-chevron-right text-[10px] text-white/70"></i>
                            </button>
                        </div>
                    </div>
                }
            })}

            // Google Maps Style Horizontal Alerts Carousel (Bottom Deck)
            <AlertsCarousel
                filtered_alerts=filtered_alerts
                selected_alert_id=selected_alert_id
                is_report_mode=is_report_mode
            />
        </main>
    }
}
