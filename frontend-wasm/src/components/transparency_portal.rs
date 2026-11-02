use leptos::*;
use crate::api::export_csv_url;
use crate::models::{EcoAlert, EcoCategory, EcoDistrict};

#[component]
pub fn TransparencyPortal(
    alerts: RwSignal<Vec<EcoAlert>>,
    show_transparency_portal: RwSignal<bool>,
    inspection_target: RwSignal<Option<EcoAlert>>,
) -> impl IntoView {
    let selected_category = create_rw_signal(None::<EcoCategory>);
    let selected_district = create_rw_signal(None::<EcoDistrict>);
    let search_query = create_rw_signal("".to_string());
    let lightbox_image = create_rw_signal(None::<(String, String)>); // (url, title)

    let solved_alerts = create_memo(move |_| {
        alerts
            .get()
            .into_iter()
            .filter(|a| a.status == "solved")
            .collect::<Vec<EcoAlert>>()
    });

    let total_solved = move || solved_alerts.get().len();

    let top_category_text = move || {
        let list = solved_alerts.get();
        if list.is_empty() {
            return "Ninguna".to_string();
        }
        let mut counts = std::collections::HashMap::new();
        for a in &list {
            *counts.entry(a.category).or_insert(0) += 1;
        }
        counts
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(cat, _)| match cat {
                EcoCategory::Mineria => "Minería ⛏️".to_string(),
                EcoCategory::Basura => "Residuos Sólidos 🗑️".to_string(),
                EcoCategory::Agua => "Recursos Hídricos 💧".to_string(),
                EcoCategory::Aire => "Calidad del Aire 🌬️".to_string(),
            })
            .unwrap_or_else(|| "General".to_string())
    };

    let filtered_solved = create_memo(move |_| {
        let cat_opt = selected_category.get();
        let dist_opt = selected_district.get();
        let query = search_query.get().trim().to_lowercase();

        solved_alerts
            .get()
            .into_iter()
            .filter(|a| {
                if let Some(cat) = cat_opt {
                    if a.category != cat {
                        return false;
                    }
                }
                if let Some(dist) = dist_opt {
                    if a.district != dist {
                        return false;
                    }
                }
                if !query.is_empty() {
                    let matches_title = a.title.to_lowercase().contains(&query);
                    let matches_desc = a.description.to_lowercase().contains(&query);
                    let matches_addr = a.address.to_lowercase().contains(&query);
                    if !matches_title && !matches_desc && !matches_addr {
                        return false;
                    }
                }
                true
            })
            .collect::<Vec<EcoAlert>>()
    });

    let download_csv = move |_| {
        let url = export_csv_url();
        let _ = js_sys::eval(&format!("window.open('{}', '_blank');", url));
    };

    view! {
        {move || show_transparency_portal.get().then(|| {
            view! {
                <div class="fixed inset-0 z-40 bg-slate-50 flex flex-col overflow-y-auto animate-in fade-in duration-200 select-none">
                    // 1. Google-Style Top Sticky Navigation Bar
                    <header class="sticky top-0 z-30 bg-white/95 backdrop-blur-md border-b border-slate-200/80 px-4 sm:px-8 py-3 flex items-center justify-between gap-4 shadow-2xs">
                        <div class="flex items-center space-x-3 min-w-0">
                            <div class="w-9 h-9 rounded-full bg-emerald-600 text-white flex items-center justify-center font-bold text-sm shadow-xs shrink-0">
                                <svg class="w-4 h-4 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.3">
                                    <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
                                </svg>
                            </div>
                            <div class="min-w-0">
                                <div class="flex items-center space-x-2">
                                    <h1 class="text-base font-bold text-slate-900 leading-tight tracking-tight">
                                        "Portal de Transparencia Ciudadana"
                                    </h1>
                                    <span class="hidden sm:inline text-[9px] uppercase font-bold tracking-wider px-2 py-0.5 rounded-full bg-emerald-50 text-emerald-800 border border-emerald-200">
                                        "Gobierno Abierto"
                                    </span>
                                </div>
                                <p class="text-xs text-slate-500 font-medium truncate">
                                    "Monitoreo de logros, mitigación y obras ambientales · Cerro de Pasco"
                                </p>
                            </div>
                        </div>

                        <div class="flex items-center space-x-2 shrink-0">
                            // Google Sheets / CSV Export Pill
                            <button
                                class="px-3.5 py-1.5 text-xs font-semibold text-slate-700 bg-white hover:bg-slate-50 border border-slate-200/90 rounded-full shadow-2xs transition flex items-center space-x-1.5 hover:border-slate-300"
                                title="Descargar registro de alertas resueltas en formato CSV"
                                on:click=download_csv
                            >
                                <svg class="w-3.5 h-3.5 text-emerald-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                                    <polyline points="7 10 12 15 17 10"/>
                                    <line x1="12" y1="15" x2="12" y2="3"/>
                                </svg>
                                <span class="hidden sm:inline">"Exportar CSV"</span>
                            </button>

                            // Google Maps Style Return Button
                            <button
                                class="px-4 py-1.5 text-xs font-semibold bg-slate-900 hover:bg-slate-800 text-white rounded-full shadow-2xs transition flex items-center space-x-1.5"
                                on:click=move |_| show_transparency_portal.set(false)
                            >
                                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <polygon points="3 6 9 3 15 6 21 3 21 18 15 21 9 18 3 21"/>
                                    <line x1="9" y1="3" x2="9" y2="18"/>
                                    <line x1="15" y1="6" x2="15" y2="21"/>
                                </svg>
                                <span>"Volver al Mapa"</span>
                            </button>
                        </div>
                    </header>

                    // 2. Main Scrollable Container
                    <main class="flex-1 max-w-5xl w-full mx-auto px-4 sm:px-6 py-6 space-y-5">
                        // Hero Banner (Google Environmental Insights aesthetic)
                        <div class="relative overflow-hidden rounded-3xl bg-gradient-to-r from-emerald-50 via-teal-50/60 to-sky-50 border border-emerald-100/90 p-6 sm:p-8 text-slate-800 shadow-2xs">
                            <div class="relative z-10 max-w-2xl space-y-2.5">
                                <span class="inline-flex items-center space-x-1.5 px-3 py-1 rounded-full text-xs font-bold bg-white text-emerald-800 border border-emerald-200/80 shadow-2xs">
                                    <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>
                                    <span>"Evidencia Física y Rendición de Cuentas"</span>
                                </span>
                                <h2 class="text-xl sm:text-2xl font-bold tracking-tight text-slate-900">
                                    "Mitigación Ambiental Abierta · Cerro de Pasco"
                                </h2>
                                <p class="text-xs sm:text-sm text-slate-600 leading-relaxed font-normal">
                                    "Supervisión pública en tiempo real de las denuncias ambientales atendidas y mitigadas por las municipalidades de Chaupimarca, Yanacancha y Simón Bolívar. Verifica el antes y después físico de cada trabajo con acta técnica oficial."
                                </p>
                            </div>
                            <div class="absolute -right-6 -bottom-8 opacity-10 pointer-events-none text-emerald-800">
                                <svg class="w-56 h-56" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                                    <path d="M12 22a7 7 0 0 0 7-7c0-2-1-3.9-3-5.5s-3.5-4-4-6.5c-.5 2.5-2 4.9-4 6.5C6 11.1 5 13 5 15a7 7 0 0 0 7 7z"/>
                                </svg>
                            </div>
                        </div>

                        // 3. Metrics Overview Cards (Google Material 3 Elevated Cards)
                        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3.5">
                            // Metric 1: Casos Resueltos
                            <div class="p-4 rounded-2xl bg-white border border-slate-200/80 shadow-2xs flex items-center space-x-3.5 hover:shadow-xs transition">
                                <div class="w-11 h-11 rounded-full bg-emerald-50 text-emerald-700 border border-emerald-100 flex items-center justify-center shrink-0">
                                    <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4">
                                        <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                                        <polyline points="22 4 12 14.01 9 11.01"></polyline>
                                    </svg>
                                </div>
                                <div class="min-w-0">
                                    <p class="text-[11px] font-semibold text-slate-500 uppercase tracking-wider">"Casos Resueltos"</p>
                                    <h3 class="text-2xl font-bold text-slate-900 font-mono mt-0.5">
                                        {total_solved}
                                    </h3>
                                    <p class="text-[10px] text-emerald-700 font-semibold truncate">
                                        "100% con evidencia pericial"
                                    </p>
                                </div>
                            </div>

                            // Metric 2: Tiempo de Respuesta
                            <div class="p-4 rounded-2xl bg-white border border-slate-200/80 shadow-2xs flex items-center space-x-3.5 hover:shadow-xs transition">
                                <div class="w-11 h-11 rounded-full bg-sky-50 text-sky-700 border border-sky-100 flex items-center justify-center shrink-0">
                                    <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4">
                                        <circle cx="12" cy="12" r="10"></circle>
                                        <polyline points="12 6 12 12 16 14"></polyline>
                                    </svg>
                                </div>
                                <div class="min-w-0">
                                    <p class="text-[11px] font-semibold text-slate-500 uppercase tracking-wider">"Tiempo de Mitigación"</p>
                                    <h3 class="text-2xl font-bold text-slate-900 font-mono mt-0.5">
                                        "1.8 días"
                                    </h3>
                                    <p class="text-[10px] text-sky-700 font-semibold truncate">
                                        "Promedio provincial activo"
                                    </p>
                                </div>
                            </div>

                            // Metric 3: Mayor Foco de Atención
                            <div class="p-4 rounded-2xl bg-white border border-slate-200/80 shadow-2xs flex items-center space-x-3.5 hover:shadow-xs transition">
                                <div class="w-11 h-11 rounded-full bg-amber-50 text-amber-700 border border-amber-100 flex items-center justify-center shrink-0">
                                    <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4">
                                        <circle cx="12" cy="12" r="10"></circle>
                                        <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"></polygon>
                                    </svg>
                                </div>
                                <div class="min-w-0">
                                    <p class="text-[11px] font-semibold text-slate-500 uppercase tracking-wider">"Foco Prioritario"</p>
                                    <h3 class="text-base font-bold text-slate-900 truncate mt-0.5">
                                        {top_category_text}
                                    </h3>
                                    <p class="text-[10px] text-amber-700 font-semibold truncate">
                                        "Sector con más intervenciones"
                                    </p>
                                </div>
                            </div>
                        </div>

                        // 4. Google Search & Filter Bar (Material 3 style)
                        <div class="p-3.5 bg-white rounded-2xl border border-slate-200/80 shadow-2xs space-y-3">
                            <div class="flex flex-col sm:flex-row gap-2.5 items-center justify-between">
                                // Search Input
                                <div class="relative w-full sm:w-80">
                                    <svg class="w-4 h-4 text-slate-400 absolute left-3 top-1/2 transform -translate-y-1/2 pointer-events-none" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                        <circle cx="11" cy="11" r="8"></circle>
                                        <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                                    </svg>
                                    <input
                                        type="text"
                                        placeholder="Buscar en casos resueltos..."
                                        class="w-full pl-9 pr-8 py-1.5 text-xs bg-slate-50 border border-slate-200 rounded-full focus:bg-white focus:outline-none focus:ring-2 focus:ring-emerald-100 focus:border-emerald-500 transition"
                                        prop:value=move || search_query.get()
                                        on:input=move |ev| search_query.set(event_target_value(&ev))
                                    />
                                    {move || (!search_query.get().is_empty()).then(|| view! {
                                        <button
                                            class="absolute right-2.5 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600 p-0.5 rounded-full"
                                            on:click=move |_| search_query.set("".to_string())
                                        >
                                            <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                                                <line x1="18" y1="6" x2="6" y2="18"></line>
                                                <line x1="6" y1="6" x2="18" y2="18"></line>
                                            </svg>
                                        </button>
                                    })}
                                </div>

                                // Category Filter Chips (Google M3)
                                <div class="flex items-center space-x-1.5 overflow-x-auto w-full sm:w-auto text-xs pb-1 sm:pb-0 no-scrollbar">
                                    <button
                                        class=move || {
                                            if selected_category.get().is_none() {
                                                "px-3 py-1 rounded-full bg-slate-900 text-white font-semibold transition shadow-2xs"
                                            } else {
                                                "px-3 py-1 rounded-full bg-slate-100 hover:bg-slate-200 text-slate-600 font-medium transition"
                                            }
                                        }
                                        on:click=move |_| selected_category.set(None)
                                    >
                                        "Todas"
                                    </button>
                                    {EcoCategory::all().iter().map(|&cat| {
                                        let is_sel = move || selected_category.get() == Some(cat);
                                        view! {
                                            <button
                                                class=move || {
                                                    if is_sel() {
                                                        "px-3 py-1 rounded-full bg-slate-900 text-white font-semibold transition shadow-2xs"
                                                    } else {
                                                        "px-3 py-1 rounded-full bg-slate-100 hover:bg-slate-200 text-slate-600 font-medium transition"
                                                    }
                                                }
                                                on:click=move |_| selected_category.set(Some(cat))
                                            >
                                                {cat.icon()} " " {cat.display_name()}
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                            </div>

                            // District Filter Chips
                            <div class="flex items-center space-x-1.5 overflow-x-auto text-xs pt-2 border-t border-slate-100 no-scrollbar">
                                <span class="text-[10px] font-bold text-slate-400 uppercase tracking-wider mr-1">
                                    "Distrito:"
                                </span>
                                <button
                                    class=move || {
                                        if selected_district.get().is_none() {
                                            "px-2.5 py-0.5 rounded-full bg-emerald-700 text-white font-semibold transition shadow-2xs text-[11px]"
                                        } else {
                                            "px-2.5 py-0.5 rounded-full bg-slate-100 hover:bg-slate-200 text-slate-600 font-medium transition text-[11px]"
                                        }
                                    }
                                    on:click=move |_| selected_district.set(None)
                                >
                                    "Toda la Provincia"
                                </button>
                                {EcoDistrict::all().iter().map(|&dist| {
                                    let is_sel = move || selected_district.get() == Some(dist);
                                    view! {
                                        <button
                                            class=move || {
                                                if is_sel() {
                                                    "px-2.5 py-0.5 rounded-full bg-emerald-700 text-white font-semibold transition shadow-2xs text-[11px]"
                                                } else {
                                                    "px-2.5 py-0.5 rounded-full bg-slate-100 hover:bg-slate-200 text-slate-600 font-medium transition text-[11px]"
                                                }
                                            }
                                            on:click=move |_| selected_district.set(Some(dist))
                                        >
                                            {dist.display_name()}
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                        </div>

                        // 5. Gallery List of Solved Cases
                        <div class="flex items-center justify-between pt-1">
                            <div>
                                <h3 class="text-sm font-bold text-slate-900">
                                    "Evidencia Física y Actas Municipales"
                                </h3>
                                <p class="text-[11px] text-slate-500">
                                    "Casos pericialmente atendidos y cerrados con fiscalización comunal"
                                </p>
                            </div>
                            <span class="text-[11px] font-semibold text-slate-500 font-mono bg-slate-100 px-2 py-0.5 rounded-full">
                                {move || format!("{} casos", filtered_solved.get().len())}
                            </span>
                        </div>

                        {move || {
                            let list = filtered_solved.get();
                            if list.is_empty() {
                                view! {
                                    <div class="p-12 text-center bg-white rounded-3xl border border-slate-200/80 shadow-2xs space-y-3">
                                        <div class="w-12 h-12 rounded-full bg-slate-100 text-slate-400 flex items-center justify-center mx-auto text-xl">
                                            "🔍"
                                        </div>
                                        <h4 class="text-sm font-bold text-slate-800">"No se encontraron casos resueltos"</h4>
                                        <p class="text-xs text-slate-500 max-w-sm mx-auto">
                                            "Prueba quitando los filtros o realizando otra búsqueda de incidencias."
                                        </p>
                                    </div>
                                }.into_view()
                            } else {
                                view! {
                                    <div class="space-y-5">
                                        {list.into_iter().map(|alert| {
                                            let alert_title = alert.title.clone();
                                            let alert_title_for_lightbox_1 = alert.title.clone();
                                            let alert_title_for_lightbox_2 = alert.title.clone();

                                            let muni_name = match alert.district {
                                                EcoDistrict::Yanacancha => "Muni Distrital de Yanacancha",
                                                EcoDistrict::SimonBolivar => "Muni Distrital de Simón Bolívar",
                                                EcoDistrict::Chaupimarca => "Muni Provincial de Pasco",
                                            };

                                            let resolved_date = alert.resolved_at.clone().unwrap_or_else(|| "Reciente".to_string());
                                            let before_img = alert.image_url.clone();
                                            let after_img = alert.resolution_image_url.clone();

                                            view! {
                                                <div class="bg-white rounded-3xl border border-slate-200/90 p-5 sm:p-6 shadow-2xs hover:shadow-sm transition-all space-y-4">
                                                    // Header Row: Category, Resuelto Badge, Municipality
                                                    <div class="flex flex-wrap items-center justify-between gap-2 border-b border-slate-100 pb-3">
                                                        <div class="flex items-center space-x-2">
                                                            <span class=format!("px-2.5 py-0.5 rounded-full text-xs font-semibold border {}", alert.category.color_class())>
                                                                {alert.category.icon()} " " {alert.category.display_name()}
                                                            </span>

                                                            <span class="px-2.5 py-0.5 rounded-full text-xs font-bold bg-emerald-50 text-emerald-800 border border-emerald-200/80 flex items-center space-x-1">
                                                                <svg class="w-3.5 h-3.5 text-emerald-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                                                                    <polyline points="20 6 9 17 4 12"></polyline>
                                                                </svg>
                                                                <span>{format!("RESUELTO: {}", resolved_date)}</span>
                                                            </span>
                                                        </div>

                                                        <span class="text-xs font-semibold text-slate-500 flex items-center space-x-1">
                                                            <span>"🏛️"</span>
                                                            <span>{muni_name}</span>
                                                        </span>
                                                    </div>

                                                    // Title, Description & Address
                                                    <div>
                                                        <h4 class="text-base font-bold text-slate-900 leading-snug">
                                                            {alert_title}
                                                        </h4>
                                                        <p class="text-xs text-slate-600 mt-1 leading-relaxed">
                                                            {alert.description.clone()}
                                                        </p>
                                                        <div class="flex items-center space-x-1.5 mt-2 text-slate-400 text-xs">
                                                            <svg class="w-3.5 h-3.5 text-slate-400 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                                <path d="M20 10c0 6-8 12-8 12s-8-6-8-12a8 8 0 0 1 16 0Z"/>
                                                                <circle cx="12" cy="10" r="3"/>
                                                            </svg>
                                                            <span class="truncate">{alert.address.clone()}</span>
                                                        </div>
                                                    </div>

                                                    // Side-by-Side "Antes vs Después" Photo Viewer
                                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-3.5 pt-1">
                                                        // ANTES (Denuncia Ciudadana)
                                                        <div class="space-y-1.5">
                                                            <div class="flex items-center justify-between text-xs font-bold text-slate-600">
                                                                <span>"ANTES: Denuncia Inicial"</span>
                                                                <span class="text-[10px] text-slate-400 font-normal">"Evidencia pericial"</span>
                                                            </div>
                                                            <div class="relative h-52 rounded-2xl overflow-hidden bg-slate-100 border border-slate-200 group">
                                                                {if let Some(ref img_url) = before_img {
                                                                    let u = img_url.clone();
                                                                    let t = alert_title_for_lightbox_1.clone();
                                                                    view! {
                                                                        <img
                                                                            src=u.clone()
                                                                            class="w-full h-full object-cover transition duration-300 group-hover:scale-105 cursor-zoom-in"
                                                                            alt="Fotografía antes de la mitigación"
                                                                            on:click=move |_| lightbox_image.set(Some((u.clone(), format!("Antes: {}", t))))
                                                                        />
                                                                        <div class="absolute bottom-2 left-2 px-2 py-0.5 rounded-full bg-slate-900/75 text-white text-[10px] font-semibold flex items-center space-x-1">
                                                                            <span>"🔍 Clic para ampliar"</span>
                                                                        </div>
                                                                    }.into_view()
                                                                } else {
                                                                    view! {
                                                                        <div class="w-full h-full flex flex-col items-center justify-center text-slate-400 text-xs space-y-1">
                                                                            <svg class="w-8 h-8 text-slate-300" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                                                                                <rect width="18" height="18" x="3" y="3" rx="2" ry="2"/>
                                                                                <circle cx="9" cy="9" r="2"/>
                                                                                <path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/>
                                                                            </svg>
                                                                            <span>"Sin foto inicial adjunta"</span>
                                                                        </div>
                                                                    }.into_view()
                                                                }}
                                                            </div>
                                                        </div>

                                                        // DESPUÉS (Remediación de la Municipalidad)
                                                        <div class="space-y-1.5">
                                                            <div class="flex items-center justify-between text-xs font-bold text-emerald-800">
                                                                <span>"DESPUÉS: Remediación Verificada"</span>
                                                                <span class="text-[10px] font-bold text-emerald-700 bg-emerald-50 px-2 py-0.5 rounded-full border border-emerald-200">
                                                                    "✅ Verificado"
                                                                </span>
                                                            </div>
                                                            <div class="relative h-52 rounded-2xl overflow-hidden bg-emerald-50/50 border-2 border-emerald-500/30 group">
                                                                {if let Some(ref res_img) = after_img {
                                                                    let u = res_img.clone();
                                                                    let t = alert_title_for_lightbox_2.clone();
                                                                    view! {
                                                                        <img
                                                                            src=u.clone()
                                                                            class="w-full h-full object-cover transition duration-300 group-hover:scale-105 cursor-zoom-in"
                                                                            alt="Fotografía después de la remediación"
                                                                            on:click=move |_| lightbox_image.set(Some((u.clone(), format!("Después: {}", t))))
                                                                        />
                                                                        <div class="absolute bottom-2 left-2 px-2.5 py-0.5 rounded-full bg-emerald-950/80 text-emerald-100 text-[10px] font-semibold flex items-center space-x-1 shadow-xs">
                                                                            <svg class="w-3 h-3 text-emerald-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                                                                                <polyline points="20 6 9 17 4 12"></polyline>
                                                                            </svg>
                                                                            <span>"Evidencia de Limpieza"</span>
                                                                        </div>
                                                                    }.into_view()
                                                                } else {
                                                                    view! {
                                                                        <div class="w-full h-full flex flex-col items-center justify-center text-slate-400 text-xs space-y-1">
                                                                            <svg class="w-8 h-8 text-slate-300" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                                                                                <path d="M14.5 4h-5L7 7H4a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-3l-2.5-3z"/>
                                                                                <circle cx="12" cy="13" r="3"/>
                                                                            </svg>
                                                                            <span>"Evidencia fotográfica en proceso de carga"</span>
                                                                        </div>
                                                                    }.into_view()
                                                                }}
                                                            </div>
                                                        </div>
                                                    </div>

                                                    // Municipal Resolution Comment (Google Callout Card)
                                                    {alert.resolution_comment.clone().map(|comm| view! {
                                                        <div class="p-3.5 rounded-2xl bg-emerald-50/70 border border-emerald-200/80 flex items-start space-x-2.5 text-xs">
                                                            <div class="w-6 h-6 rounded-full bg-emerald-100 text-emerald-700 flex items-center justify-center shrink-0 mt-0.5 font-bold">
                                                                "🏛️"
                                                            </div>
                                                            <div class="space-y-0.5 flex-1">
                                                                <p class="text-[10px] font-bold text-emerald-900 uppercase tracking-wider">
                                                                    "Informe Técnico Municipal:"
                                                                </p>
                                                                <p class="text-xs text-emerald-950 leading-relaxed font-medium">
                                                                    {comm}
                                                                </p>
                                                            </div>
                                                        </div>
                                                    })}

                                                    // Official Acta Document Action
                                                    <div class="pt-1 flex items-center justify-end">
                                                        <button
                                                            class="px-3.5 py-1.5 rounded-xl bg-slate-900 hover:bg-slate-800 text-white font-semibold text-xs flex items-center space-x-2 transition shadow-xs cursor-pointer"
                                                            title="Ver e imprimir Acta Oficial de Fiscalización e Intervención"
                                                            on:click={
                                                                let a_clone = alert.clone();
                                                                move |_| inspection_target.set(Some(a_clone.clone()))
                                                            }
                                                        >
                                                            <i class="fa-solid fa-file-contract text-sky-400 text-xs"></i>
                                                            <span>"Ver Acta Oficial e Imprimir"</span>
                                                        </button>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_view()
                            }
                        }}
                    </main>

                    // Full-screen Image Lightbox Modal
                    {move || lightbox_image.get().map(|(url, title)| {
                        view! {
                            <div
                                class="fixed inset-0 z-50 bg-black/90 backdrop-blur-md flex flex-col items-center justify-center p-4 animate-in fade-in duration-150"
                                on:click=move |_| lightbox_image.set(None)
                            >
                                <div class="relative max-w-4xl max-h-[85vh] w-full flex flex-col items-center space-y-2">
                                    <div class="flex justify-between items-center w-full text-white text-xs px-2">
                                        <span class="font-bold truncate max-w-md">{title}</span>
                                        <button
                                            class="p-1 rounded-full bg-white/10 hover:bg-white/20 text-white transition"
                                            on:click=move |_| lightbox_image.set(None)
                                        >
                                            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                <line x1="18" y1="6" x2="6" y2="18"></line>
                                                <line x1="6" y1="6" x2="18" y2="18"></line>
                                            </svg>
                                        </button>
                                    </div>
                                    <img
                                        src=url
                                        class="max-w-full max-h-[80vh] rounded-2xl object-contain shadow-2xl border border-white/20"
                                        alt="Evidencia ampliada"
                                        on:click=move |e| e.stop_propagation()
                                    />
                                </div>
                            </div>
                        }
                    })}
                </div>
            }
        })}
    }
}
