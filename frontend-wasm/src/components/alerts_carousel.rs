use leptos::*;
use crate::models::{EcoAlert, EcoSeverity};

#[component]
pub fn AlertsCarousel(
    filtered_alerts: Memo<Vec<EcoAlert>>,
    selected_alert_id: RwSignal<Option<String>>,
    is_report_mode: RwSignal<bool>,
) -> impl IntoView {
    let (is_mobile_init, _) = create_signal({
        if let Some(win) = web_sys::window() {
            win.inner_width().map(|w| w.as_f64().unwrap_or(1200.0) < 768.0).unwrap_or(false)
        } else {
            false
        }
    });
    let is_collapsed = create_rw_signal(is_mobile_init.get());
    let visible_count = create_rw_signal(10); // Progressive batch loading: 10 items at a time

    // Reset pagination when the filtered alerts change
    create_effect(move |_| {
        let _ = filtered_alerts.get();
        visible_count.set(10);
    });

    // Initialize touch and mouse drag-and-swipe whenever the carousel mounts or renders
    create_effect(move |_| {
        let _ = is_collapsed.get();
        let _ = visible_count.get();
        let _ = js_sys::eval(
            "setTimeout(() => window.initAlertsCarouselDrag && window.initAlertsCarouselDrag(), 60);"
        );
    });

    let scroll_left = move |_| {
        let _ = js_sys::eval(
            "document.getElementById('alerts-carousel-container')?.scrollBy({ left: -340, behavior: 'smooth' });"
        );
    };

    let scroll_right = move |_| {
        let _ = js_sys::eval(
            "document.getElementById('alerts-carousel-container')?.scrollBy({ left: 340, behavior: 'smooth' });"
        );
    };

    let fly_to_alert = move |a: &EcoAlert| {
        let lat = a.latitude;
        let lng = a.longitude;
        let id = a.id.clone();
        selected_alert_id.set(Some(id));
        let _ = js_sys::eval(&format!("window.ecoMap.flyTo({}, {}, 16.5);", lat, lng));
    };

    let total_count = move || filtered_alerts.get().len();
    let critical_count = move || {
        filtered_alerts.get().iter().filter(|a| a.severity == EcoSeverity::Critico).count()
    };

    view! {
        // Hide carousel in report pin mode so map center remains clear
        {move || (!is_report_mode.get()).then(|| {
            view! {
                <div class="absolute bottom-[3.75rem] md:bottom-2.5 left-2.5 right-2.5 md:left-3 md:right-3 z-20 pointer-events-none flex flex-col items-start gap-1 select-none transition-all duration-300">
                    // 1. Carousel Header Controls (Unified with Navigation Controls)
                    <div class="flex items-center justify-between w-full pointer-events-auto">
                        // Compact status & toggle pill
                        <button
                            class="bg-white/95 backdrop-blur-md px-3 py-1.5 rounded-xl shadow-[0_2px_8px_rgba(60,64,67,0.14)] border border-slate-200/90 text-slate-800 hover:text-emerald-700 flex items-center space-x-1.5 text-xs font-semibold transition hover:scale-[1.01] cursor-pointer"
                            on:click=move |_| is_collapsed.update(|v| *v = !*v)
                            title="Mostrar / Minimizar denuncias en el mapa"
                        >
                            <i class="fa-solid fa-map-location-dot text-emerald-600 text-xs"></i>
                            <span class="font-bold tracking-tight text-slate-900 text-[11px] sm:text-xs">"Denuncias en esta zona"</span>
                            <span class="px-1.5 py-0.2 rounded-md bg-emerald-50 text-[10px] text-emerald-800 border border-emerald-100 font-mono font-bold">
                                {total_count}
                            </span>
                            {move || (critical_count() > 0).then(|| view! {
                                <span class="px-1.5 py-0.2 rounded-md bg-red-50 text-[10px] text-red-700 border border-red-100 font-mono font-bold">
                                    {format!("{} Críticas", critical_count())}
                                </span>
                            })}
                            <i class=move || {
                                if is_collapsed.get() {
                                    "fa-solid fa-chevron-up text-slate-400 text-[10px] ml-0.5 transition-transform"
                                } else {
                                    "fa-solid fa-chevron-down text-slate-400 text-[10px] ml-0.5 transition-transform"
                                }
                            }></i>
                        </button>

                        // Quick Navigation Arrows & Counter (Header-integrated)
                        {move || (!is_collapsed.get() && total_count() > 1).then(|| view! {
                            <div class="flex items-center space-x-1 bg-white/95 backdrop-blur-md p-1 rounded-xl shadow-[0_2px_8px_rgba(60,64,67,0.14)] border border-slate-200/90 text-slate-600">
                                <span class="px-1.5 text-[10px] font-mono font-bold text-slate-500 hidden sm:inline">
                                    {move || format!("{} de {}", visible_count.get().min(total_count()), total_count())}
                                </span>
                                <button
                                    class="w-6 h-6 rounded-lg hover:bg-slate-100 text-slate-600 hover:text-slate-900 flex items-center justify-center transition cursor-pointer"
                                    title="Desplazar hacia la izquierda"
                                    on:click=scroll_left
                                >
                                    <i class="fa-solid fa-chevron-left text-[10px]"></i>
                                </button>
                                <button
                                    class="w-6 h-6 rounded-lg hover:bg-slate-100 text-slate-600 hover:text-slate-900 flex items-center justify-center transition cursor-pointer"
                                    title="Desplazar hacia la derecha"
                                    on:click=scroll_right
                                >
                                    <i class="fa-solid fa-chevron-right text-[10px]"></i>
                                </button>
                            </div>
                        })}
                    </div>

                    // 2. Horizontal Cards Track (Optimized batch rendering + Touch/Drag enabled)
                    {move || (!is_collapsed.get()).then(|| {
                        let all_alerts = filtered_alerts.get();
                        let total_len = all_alerts.len();
                        let current_limit = visible_count.get();
                        let current_batch = all_alerts.into_iter().take(current_limit).collect::<Vec<_>>();
                        let has_more = total_len > current_limit;
                        let remaining = total_len.saturating_sub(current_limit);

                        view! {
                            <div class="w-full pointer-events-auto">
                                <div
                                    id="alerts-carousel-container"
                                    class="w-full flex items-center space-x-2.5 overflow-x-auto pb-1 pt-0.5 scroll-smooth cursor-grab active:cursor-grabbing touch-pan-x animate-in fade-in slide-in-from-bottom-2 duration-200 no-scrollbar"
                                >
                                    {if current_batch.is_empty() {
                                        view! {
                                            <div class="px-3.5 py-2 rounded-xl bg-white/95 backdrop-blur-md border border-slate-200/90 shadow-xs text-xs text-slate-600 flex items-center space-x-2">
                                                <i class="fa-solid fa-magnifying-glass text-slate-400"></i>
                                                <span>"No hay denuncias que coincidan con los filtros seleccionados."</span>
                                            </div>
                                        }.into_view()
                                    } else {
                                        view! {
                                            <div class="flex items-center space-x-2.5">
                                                {current_batch.into_iter().map(|alert| {
                                                    let a = alert.clone();
                                                    let aid = alert.id.clone();
                                                    let (sev_class, sev_label) = match alert.severity {
                                                        EcoSeverity::Critico => ("bg-red-600 text-white", "Crítico"),
                                                        EcoSeverity::Medio => ("bg-amber-500 text-white", "Medio"),
                                                        EcoSeverity::Bajo => ("bg-emerald-600 text-white", "Bajo"),
                                                    };

                                                    let created_date = alert.created_at.split('T').next().unwrap_or(&alert.created_at).to_string();
                                                    let title = alert.title.clone();
                                                    let address = alert.address.clone();
                                                    let district_name = alert.district.display_name();
                                                    let conf_count = alert.confirmations_count;
                                                    let cat_icon = alert.category.icon();
                                                    let cat_name = alert.category.display_name();
                                                    let img_opt = alert.image_url.clone();

                                                    let is_active = move || selected_alert_id.get() == Some(aid.clone());

                                                    view! {
                                                        <div
                                                            class=move || {
                                                                if is_active() {
                                                                    "w-64 sm:w-72 shrink-0 bg-white rounded-xl border-2 border-emerald-500 shadow-[0_4px_16px_rgba(16,185,129,0.2)] p-2.5 cursor-pointer transition-all hover:scale-[1.01] flex space-x-2.5"
                                                                } else {
                                                                    "w-64 sm:w-72 shrink-0 bg-white/95 backdrop-blur-md rounded-xl border border-slate-200/90 shadow-[0_2px_10px_rgba(60,64,67,0.12)] p-2.5 cursor-pointer transition-all hover:border-emerald-300 hover:shadow-md hover:scale-[1.01] flex space-x-2.5"
                                                                }
                                                            }
                                                            on:click={
                                                                let a_clone = a.clone();
                                                                move |_| fly_to_alert(&a_clone)
                                                            }
                                                        >
                                                            // Compact Thumbnail
                                                            <div class="w-13 h-13 sm:w-14 sm:h-14 rounded-lg overflow-hidden shrink-0 bg-slate-100 border border-slate-200/80 flex items-center justify-center relative">
                                                                {if let Some(ref url) = img_opt {
                                                                    let u = url.clone();
                                                                    view! {
                                                                        <img src=u alt="Foto de la denuncia" class="w-full h-full object-cover" />
                                                                    }.into_view()
                                                                } else {
                                                                    view! {
                                                                        <div class="flex flex-col items-center justify-center text-slate-400">
                                                                            <span class="text-lg">{cat_icon}</span>
                                                                        </div>
                                                                    }.into_view()
                                                                }}
                                                            </div>

                                                            // Compact Content Details
                                                            <div class="flex-1 min-w-0 space-y-0.5">
                                                                <div class="flex items-center justify-between text-[10px]">
                                                                    <div class="flex items-center space-x-1">
                                                                        <span class=format!("px-1.5 py-0.5 rounded-full font-bold text-[9px] text-white flex items-center gap-1 shadow-2xs {}", sev_class)>
                                                                            {match alert.severity {
                                                                                EcoSeverity::Critico => view! {
                                                                                    <svg class="w-2.5 h-2.5 text-white shrink-0" viewBox="0 0 20 20" fill="currentColor">
                                                                                        <path fill-rule="evenodd" d="M8.485 2.495c.673-1.167 2.357-1.167 3.03 0l6.28 10.875c.673 1.167-.17 2.625-1.516 2.625H3.72c-2.309 0-3.752-2.5-2.598-4.5L9.4 3.003ZM10 5a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-1.5 0v-3.5A.75.75 0 0 1 10 5Zm0 9a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z" clip-rule="evenodd" />
                                                                                    </svg>
                                                                                }.into_view(),
                                                                                EcoSeverity::Medio => view! {
                                                                                    <svg class="w-2.5 h-2.5 text-white shrink-0" viewBox="0 0 20 20" fill="currentColor">
                                                                                        <path fill-rule="evenodd" d="M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0Zm-8-5a.75.75 0 0 1 .75.75v4.5a.75.75 0 0 1-1.5 0v-4.5A.75.75 0 0 1 10 5Zm0 10a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z" clip-rule="evenodd" />
                                                                                    </svg>
                                                                                }.into_view(),
                                                                                EcoSeverity::Bajo => view! {
                                                                                    <svg class="w-2.5 h-2.5 text-white shrink-0" viewBox="0 0 20 20" fill="currentColor">
                                                                                        <path fill-rule="evenodd" d="M10 18a8 8 0 1 0 0-16 8 8 0 0 0 0 16Zm3.857-9.809a.75.75 0 0 0-1.214-.882l-3.483 4.79-1.88-1.88a.75.75 0 1 0-1.06 1.061l2.5 2.5a.75.75 0 0 0 1.137-.089l4-5.5Z" clip-rule="evenodd" />
                                                                                    </svg>
                                                                                }.into_view(),
                                                                            }}
                                                                            <span>{sev_label}</span>
                                                                        </span>
                                                                        <span class="text-slate-500 font-medium truncate max-w-[70px] text-[10px]">
                                                                            {cat_name}
                                                                        </span>
                                                                    </div>
                                                                    <span class="text-slate-400 font-mono text-[9px]">
                                                                        {created_date}
                                                                    </span>
                                                                </div>

                                                                <h4 class="text-[11px] sm:text-xs font-bold text-slate-900 line-clamp-1 leading-tight">
                                                                    {title}
                                                                </h4>

                                                                <p class="text-[9.5px] text-slate-500 line-clamp-1 flex items-center space-x-1">
                                                                    <i class="fa-solid fa-location-dot text-slate-400 text-[8.5px] shrink-0"></i>
                                                                    <span class="truncate">{address}</span>
                                                                </p>

                                                                <div class="flex items-center justify-between pt-0.5 text-[9px] text-slate-400">
                                                                    <span class="truncate">"📍 " {district_name}</span>
                                                                    <span class="font-semibold text-sky-700 bg-sky-50 px-1 py-0.2 rounded text-[8.5px] shrink-0">
                                                                        {format!("👥 {} respaldos", conf_count)}
                                                                    </span>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    }
                                                }).collect_view()}

                                                // 3. Lazy Loading "Cargar más denuncias (+10)" Card
                                                {has_more.then(|| {
                                                    view! {
                                                        <div
                                                            class="w-48 sm:w-52 shrink-0 bg-white/95 backdrop-blur-md rounded-xl border-2 border-dashed border-emerald-400 hover:border-emerald-600 hover:bg-emerald-50/60 p-2.5 cursor-pointer transition-all flex flex-col items-center justify-center text-center space-y-1.5 shadow-[0_2px_8px_rgba(60,64,67,0.08)] group select-none"
                                                            on:click=move |_| {
                                                                visible_count.update(|n| *n += 10);
                                                                let _ = js_sys::eval(
                                                                    "setTimeout(() => { document.getElementById('alerts-carousel-container')?.scrollBy({ left: 300, behavior: 'smooth' }); window.initAlertsCarouselDrag && window.initAlertsCarouselDrag(); }, 60);"
                                                                );
                                                            }
                                                        >
                                                            <div class="w-8 h-8 rounded-lg bg-emerald-100 text-emerald-700 flex items-center justify-center group-hover:scale-110 transition-transform shadow-2xs">
                                                                <i class="fa-solid fa-plus text-xs"></i>
                                                            </div>
                                                            <div class="space-y-0.2">
                                                                <p class="text-[11px] font-bold text-slate-800">"Cargar más"</p>
                                                                <p class="text-[9px] text-emerald-700 font-bold font-mono">
                                                                    {format!("+{} restantes", remaining)}
                                                                </p>
                                                            </div>
                                                        </div>
                                                    }
                                                })}
                                            </div>
                                        }.into_view()
                                    }}
                                </div>
                            </div>
                        }
                    })}
                </div>
            }
        })}
    }
}
