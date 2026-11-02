use leptos::*;
use crate::geo::haversine_distance_meters;
use crate::models::{WastePoint, WasteType};

fn format_distance(meters: f64) -> String {
    if meters < 1000.0 {
        format!("{:.0} m", meters)
    } else {
        format!("{:.1} km", meters / 1000.0)
    }
}

#[component]
pub fn WasteListModal(
    waste_points: RwSignal<Vec<WastePoint>>,
    show_waste_list_modal: RwSignal<bool>,
    selected_waste_point_id: RwSignal<Option<String>>,
    show_waste_layer: RwSignal<bool>,
    current_gps_or_center: RwSignal<(f64, f64)>,
) -> impl IntoView {
    let selected_filter = create_rw_signal(None::<WasteType>);

    view! {
        {move || show_waste_list_modal.get().then(|| {
            let (user_lat, user_lng) = current_gps_or_center.get();
            let mut list = waste_points.get();

            // Sort by proximity
            list.sort_by(|a, b| {
                let dist_a = haversine_distance_meters(user_lat, user_lng, a.latitude, a.longitude);
                let dist_b = haversine_distance_meters(user_lat, user_lng, b.latitude, b.longitude);
                dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
            });

            // Filter if requested
            let filter = selected_filter.get();
            let filtered_list: Vec<WastePoint> = list
                .into_iter()
                .filter(|p| filter.map_or(true, |f| p.waste_type == f))
                .collect();

            view! {
                <div class="fixed inset-0 bg-slate-900/50 backdrop-blur-sm flex items-end sm:items-center justify-center p-0 sm:p-4 z-50 animate-in fade-in duration-150">
                    <div class="bg-white w-full sm:max-w-lg rounded-t-3xl sm:rounded-2xl border border-slate-200/90 p-5 shadow-[0_24px_64px_rgba(0,0,0,0.18)] max-h-[88vh] flex flex-col space-y-4">
                        // Mobile Drag Pill
                        <div class="w-12 h-1.5 bg-slate-300 rounded-full mx-auto -mt-1 mb-1 sm:hidden"></div>

                        // Header
                        <div class="flex items-center justify-between pb-3 border-b border-slate-100">
                            <div class="flex items-center space-x-2.5">
                                <div class="w-9 h-9 rounded-xl bg-emerald-50 text-emerald-700 border border-emerald-100 flex items-center justify-center">
                                    <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                        <path d="M7 19H4.815a1.83 1.83 0 0 1-1.57-.881 1.785 1.785 0 0 1-.004-1.784L7.196 9.5"/>
                                        <path d="M11 19h8.2a1.8 1.8 0 0 0 1.573-.896 1.79 1.79 0 0 0 0-1.792L16.8 9.5"/>
                                        <path d="m15.5 13-3.5 6-3.5-6"/>
                                        <path d="m2 9 3-5 3 5"/>
                                        <path d="M14 4h5.5a1.8 1.8 0 0 1 1.57.88 1.79 1.79 0 0 1 .005 1.79L18.5 12"/>
                                    </svg>
                                </div>
                                <div>
                                    <h3 class="text-sm font-bold text-slate-900 leading-tight">
                                        "Botaderos y Puntos de Acopio"
                                    </h3>
                                    <p class="text-[11px] text-slate-500">
                                        "Ordenados por cercanía a tu ubicación en Cerro de Pasco"
                                    </p>
                                </div>
                            </div>
                            <button
                                class="text-slate-400 hover:text-slate-600 p-1.5 rounded-lg transition"
                                on:click=move |_| show_waste_list_modal.set(false)
                            >
                                <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <line x1="18" y1="6" x2="6" y2="18"></line>
                                    <line x1="6" y1="6" x2="18" y2="18"></line>
                                </svg>
                            </button>
                        </div>

                        // Filter Pills
                        <div class="flex items-center space-x-1.5 overflow-x-auto pb-1 text-xs">
                            <button
                                class=move || {
                                    if selected_filter.get().is_none() {
                                        "px-2.5 py-1 rounded-lg bg-slate-900 text-white font-semibold transition"
                                    } else {
                                        "px-2.5 py-1 rounded-lg bg-slate-100 hover:bg-slate-200 text-slate-600 font-medium transition"
                                    }
                                }
                                on:click=move |_| selected_filter.set(None)
                            >
                                "Todos"
                            </button>
                            <button
                                class=move || {
                                    if selected_filter.get() == Some(WasteType::Recyclable) {
                                        "px-2.5 py-1 rounded-lg bg-emerald-700 text-white font-semibold transition"
                                    } else {
                                        "px-2.5 py-1 rounded-lg bg-slate-100 hover:bg-slate-200 text-slate-600 font-medium transition"
                                    }
                                }
                                on:click=move |_| selected_filter.set(Some(WasteType::Recyclable))
                            >
                                "♻️ Reciclables"
                            </button>
                            <button
                                class=move || {
                                    if selected_filter.get() == Some(WasteType::Organic) {
                                        "px-2.5 py-1 rounded-lg bg-amber-700 text-white font-semibold transition"
                                    } else {
                                        "px-2.5 py-1 rounded-lg bg-slate-100 hover:bg-slate-200 text-slate-600 font-medium transition"
                                    }
                                }
                                on:click=move |_| selected_filter.set(Some(WasteType::Organic))
                            >
                                "🍂 Orgánicos"
                            </button>
                            <button
                                class=move || {
                                    if selected_filter.get() == Some(WasteType::General) {
                                        "px-2.5 py-1 rounded-lg bg-slate-700 text-white font-semibold transition"
                                    } else {
                                        "px-2.5 py-1 rounded-lg bg-slate-100 hover:bg-slate-200 text-slate-600 font-medium transition"
                                    }
                                }
                                on:click=move |_| selected_filter.set(Some(WasteType::General))
                            >
                                "🗑️ General"
                            </button>
                        </div>

                        // Scrollable List
                        <div class="overflow-y-auto flex-1 space-y-2.5 pr-1">
                            {filtered_list.into_iter().map(|p| {
                                let point_id = p.id_string();
                                let pid_for_click = point_id.clone();
                                let lat = p.latitude;
                                let lng = p.longitude;
                                let dist = haversine_distance_meters(user_lat, user_lng, lat, lng);
                                let dist_str = format_distance(dist);

                                let on_select_point = move |_| {
                                    show_waste_list_modal.set(false);
                                    show_waste_layer.set(true);
                                    let _ = js_sys::eval(&format!("window.ecoMap.flyTo({}, {}, 16.5);", lat, lng));
                                    selected_waste_point_id.set(Some(pid_for_click.clone()));
                                };

                                view! {
                                    <div
                                        class="p-3 rounded-xl border border-slate-200/80 bg-slate-50/70 hover:bg-white hover:border-slate-300 transition cursor-pointer flex items-center justify-between gap-3 shadow-xs"
                                        on:click=on_select_point
                                    >
                                        <div class="flex items-start space-x-3 min-w-0">
                                            <div class={format!("w-9 h-9 rounded-xl flex items-center justify-center shrink-0 border mt-0.5 {}", p.waste_type.color_class())}>
                                                {match p.waste_type {
                                                    WasteType::Recyclable => view! {
                                                        <svg class="w-4 h-4 text-emerald-700" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                                                            <path d="M7 19H4.815a1.83 1.83 0 0 1-1.57-.881 1.785 1.785 0 0 1-.004-1.784L7.196 9.5"/>
                                                            <path d="M11 19h8.2a1.8 1.8 0 0 0 1.573-.896 1.79 1.79 0 0 0 0-1.792L16.8 9.5"/>
                                                            <path d="m15.5 13-3.5 6-3.5-6"/>
                                                            <path d="m2 9 3-5 3 5"/>
                                                        </svg>
                                                    }.into_view(),
                                                    WasteType::Organic => view! {
                                                        <svg class="w-4 h-4 text-amber-700" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                                                            <path d="M11 20A7 7 0 0 1 9.8 6.1C15.5 5 17 4.48 19 2c1 2 2 4.18 2 8 0 5.5-4.78 10-10 10Z"/>
                                                            <path d="M2 21c0-3 1.85-5.36 5.08-6C9.5 14.52 12 13 13 12"/>
                                                        </svg>
                                                    }.into_view(),
                                                    WasteType::General => view! {
                                                        <svg class="w-4 h-4 text-slate-700" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                                                            <path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/>
                                                        </svg>
                                                    }.into_view(),
                                                }}
                                            </div>
                                            <div class="min-w-0">
                                                <h4 class="text-xs font-bold text-slate-900 leading-snug truncate">
                                                    {p.name.clone()}
                                                </h4>
                                                <p class="text-[11px] text-slate-500 truncate mt-0.5">
                                                    {if !p.address.is_empty() { p.address.clone() } else { p.description.clone() }}
                                                </p>
                                                <div class="flex items-center space-x-2 mt-1">
                                                    <span class={format!("px-1.5 py-0.5 rounded text-[9px] font-bold border {}", p.fill_level.badge_class())}>
                                                        {p.fill_level.display_name()}
                                                    </span>
                                                    <span class="text-[10px] text-slate-500 truncate">
                                                        "🕒 " {p.truck_schedule.clone()}
                                                    </span>
                                                </div>
                                            </div>
                                        </div>

                                        <div class="text-right shrink-0">
                                            <span class="text-xs font-bold text-sky-700 font-mono">
                                                {dist_str}
                                            </span>
                                            <svg class="w-4 h-4 text-slate-400 mt-1 ml-auto" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                <polyline points="9 18 15 12 9 6"></polyline>
                                            </svg>
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                </div>
            }
        })}
    }
}
