use std::collections::HashSet;
use leptos::*;
use crate::models::{AuthUser, EcoAlert, EcoCategory, EcoDistrict, EcoSeverity, SyncStatus};

#[component]
pub fn Sidebar(
    alerts: RwSignal<Vec<EcoAlert>>,
    filtered_alerts: Memo<Vec<EcoAlert>>,
    selected_categories: RwSignal<HashSet<EcoCategory>>,
    selected_severities: RwSignal<HashSet<EcoSeverity>>,
    selected_district: RwSignal<Option<EcoDistrict>>,
    search_query: RwSignal<String>,
    is_report_mode: RwSignal<bool>,
    current_user: RwSignal<Option<AuthUser>>,
    show_auth_dialog: RwSignal<bool>,
    show_transparency_portal: RwSignal<bool>,
    show_authority_dashboard: RwSignal<bool>,
    #[allow(unused_variables)]
    selected_alert_id: RwSignal<Option<String>>,
    show_help_guide: RwSignal<bool>,
    status_filter: RwSignal<String>,
    days_filter: RwSignal<u32>,
    sync_status: RwSignal<SyncStatus>,
    on_manual_sync: Callback<()>,
    show_user_profile: RwSignal<bool>,
    is_collapsed: RwSignal<bool>,
) -> impl IntoView {
    let show_charts = create_rw_signal(true);

    let toggle_category = move |cat: EcoCategory| {
        selected_categories.update(|set| {
            if set.contains(&cat) {
                if set.len() > 1 {
                    set.remove(&cat);
                }
            } else {
                set.insert(cat);
            }
        });
    };

    let select_severity = move |sev: EcoSeverity| {
        selected_severities.update(|set| {
            if set.contains(&sev) {
                if set.len() > 1 {
                    set.remove(&sev);
                }
            } else {
                set.insert(sev);
            }
        });
    };

    let total_count = move || alerts.get().len();
    let filtered_count = move || filtered_alerts.get().len();
    let solved_count = move || alerts.get().iter().filter(|a| a.status == "solved").count();

    let critical_count = move || {
        alerts.get().iter().filter(|a| a.severity == EcoSeverity::Critico).count()
    };
    let medium_count = move || {
        alerts.get().iter().filter(|a| a.severity == EcoSeverity::Medio).count()
    };
    let low_count = move || {
        alerts.get().iter().filter(|a| a.severity == EcoSeverity::Bajo).count()
    };

    let critical_percent = move || {
        let tot = total_count();
        if tot == 0 { 0 } else { (critical_count() * 100) / tot }
    };
    let medium_percent = move || {
        let tot = total_count();
        if tot == 0 { 0 } else { (medium_count() * 100) / tot }
    };
    let low_percent = move || {
        let tot = total_count();
        if tot == 0 { 0 } else { (low_count() * 100) / tot }
    };

    // Category counts and percentages for Donut Chart
    let mineria_count = move || alerts.get().iter().filter(|a| a.category == EcoCategory::Mineria).count();
    let basura_count = move || alerts.get().iter().filter(|a| a.category == EcoCategory::Basura).count();
    let agua_count = move || alerts.get().iter().filter(|a| a.category == EcoCategory::Agua).count();
    let aire_count = move || alerts.get().iter().filter(|a| a.category == EcoCategory::Aire).count();

    let mineria_pct = move || { let t = total_count(); if t == 0 { 0.0 } else { (mineria_count() as f64 * 100.0) / t as f64 } };
    let basura_pct = move || { let t = total_count(); if t == 0 { 0.0 } else { (basura_count() as f64 * 100.0) / t as f64 } };
    let agua_pct = move || { let t = total_count(); if t == 0 { 0.0 } else { (agua_count() as f64 * 100.0) / t as f64 } };
    let aire_pct = move || { let t = total_count(); if t == 0 { 0.0 } else { (aire_count() as f64 * 100.0) / t as f64 } };

    view! {
        // Mobile Backdrop Overlay for off-canvas drawer
        {move || (!is_collapsed.get()).then(|| view! {
            <div
                class="fixed inset-0 bg-slate-900/60 backdrop-blur-xs z-40 md:hidden animate-in fade-in duration-200"
                on:click=move |_| is_collapsed.set(true)
            ></div>
        })}

        // Floating pill button to re-open sidebar when collapsed on desktop (Google Maps style)
        {move || is_collapsed.get().then(|| view! {
            <button
                class="hidden md:flex fixed top-3.5 left-3.5 z-30 bg-white/95 backdrop-blur-md pl-3 pr-3.5 py-2 rounded-xl shadow-[0_4px_16px_rgba(60,64,67,0.18)] border border-slate-200/90 text-slate-800 hover:bg-white hover:text-emerald-700 items-center space-x-2 text-xs font-semibold transition hover:scale-[1.02] animate-in fade-in duration-200 cursor-pointer"
                on:click=move |_| is_collapsed.set(false)
                title="Mostrar panel de EcoAlerta"
            >
                <div class="w-5 h-5 rounded-lg bg-emerald-600 text-white flex items-center justify-center text-[11px] shadow-2xs">
                    "🌿"
                </div>
                <span class="font-bold tracking-tight">"EcoAlerta"</span>
                <span class="px-1.5 py-0.5 rounded-md bg-emerald-50 text-[10px] text-emerald-700 border border-emerald-100 font-mono font-bold">
                    {move || format!("{}", filtered_count())}
                </span>
                <i class="fa-solid fa-chevron-right text-slate-400 text-xs"></i>
            </button>
        })}

        // Main Sidebar Container: Responsive off-canvas drawer on mobile, collapsible panel on desktop
        <aside
            class=move || {
                if is_collapsed.get() {
                    "fixed inset-y-0 left-0 -translate-x-full md:translate-x-0 md:static md:w-0 md:border-none md:p-0 md:overflow-hidden md:opacity-0 md:pointer-events-none transition-all duration-300 z-50 md:z-20"
                } else {
                    "fixed inset-y-0 left-0 translate-x-0 w-[88vw] sm:w-[380px] max-w-[420px] md:max-w-none md:static md:translate-x-0 md:w-[360px] lg:w-[380px] xl:w-[400px] h-full bg-white border-r border-slate-200/90 flex flex-col z-50 md:z-30 shrink-0 shadow-[8px_0_36px_rgba(0,0,0,0.18)] md:shadow-[4px_0_24px_rgba(60,64,67,0.06)] relative transition-all duration-300 select-none overflow-hidden md:overflow-visible"
                }
            }
        >
            // Collapse Chevron button on the right edge (desktop only)
            <button
                class="hidden md:flex absolute -right-3.5 top-1/2 -translate-y-1/2 w-7 h-7 rounded-full bg-white border border-slate-200/90 shadow-[0_2px_8px_rgba(60,64,67,0.22)] items-center justify-center text-slate-600 hover:text-slate-900 hover:bg-slate-50 transition-all z-40 cursor-pointer hover:scale-105"
                title="Ocultar panel lateral"
                on:click=move |_| is_collapsed.set(true)
            >
                <i class="fa-solid fa-chevron-left text-xs"></i>
            </button>

            // 1. Google-Style App Bar Header
            <div class="px-4 py-3.5 border-b border-slate-100 flex items-center justify-between gap-2.5 shrink-0">
                // Brand Left
                <div class="flex items-center space-x-2.5 min-w-0 flex-1">
                    <div class="w-9 h-9 rounded-xl bg-gradient-to-br from-emerald-500 to-emerald-700 text-white flex items-center justify-center shadow-xs shrink-0 ring-1 ring-emerald-600/20">
                        <svg class="w-5 h-5 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.3" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M11 20A7 7 0 0 1 9.8 6.1C15.5 5 17 4.48 19 2c1 2 2 4.18 2 8 0 5.5-4.78 10-10 10Z"/>
                            <path d="M2 21c0-3 1.85-5.36 5.08-6C9.5 14.52 12 13 13 12"/>
                        </svg>
                    </div>
                    <div class="min-w-0 flex-1">
                        <div class="flex items-center space-x-1.5">
                            <h1 class="text-[15px] font-extrabold text-slate-900 tracking-tight leading-none truncate">"EcoAlerta"</h1>
                            <span class="text-[9px] uppercase font-bold tracking-wider px-1.5 py-0.5 rounded-md bg-emerald-50 text-emerald-700 border border-emerald-200/80 shrink-0">
                                "Pasco"
                            </span>
                        </div>
                        <p class="text-[10.5px] text-slate-400 font-medium truncate mt-0.5">"Monitoreo Ambiental Activo"</p>
                    </div>
                </div>

                // Right Profile & Help Buttons
                <div class="flex items-center space-x-1.5 shrink-0">
                    // Help Center Button
                    <button
                        class="w-8 h-8 rounded-xl text-slate-500 hover:text-blue-600 hover:bg-blue-50/80 border border-slate-200/60 hover:border-blue-200 flex items-center justify-center transition cursor-pointer group"
                        title="Centro de Ayuda · Guía Ciudadana y SLA"
                        on:click=move |_| show_help_guide.set(true)
                    >
                        <i class="fa-solid fa-circle-question text-slate-400 group-hover:text-blue-600 text-sm transition-transform group-hover:scale-110"></i>
                    </button>

                    // If not logged in, show quick Acceder button in header
                    {move || current_user.get().is_none().then(|| view! {
                        <button
                            class="px-2.5 py-1.5 rounded-xl text-xs font-semibold text-emerald-700 bg-emerald-50 hover:bg-emerald-100 border border-emerald-200/80 transition flex items-center space-x-1 shadow-2xs cursor-pointer"
                            title="Iniciar sesión cívica o de autoridad"
                            on:click=move |_| show_auth_dialog.set(true)
                        >
                            <i class="fa-solid fa-user text-[11px]"></i>
                            <span>"Acceder"</span>
                        </button>
                    })}

                    // Mobile Close Drawer Button (Placed cleanly at the right corner)
                    <button
                        class="md:hidden w-8 h-8 rounded-xl bg-slate-100 hover:bg-slate-200 text-slate-600 hover:text-slate-900 border border-slate-200/70 transition flex items-center justify-center cursor-pointer shadow-2xs"
                        title="Cerrar panel"
                        on:click=move |_| is_collapsed.set(true)
                    >
                        <i class="fa-solid fa-xmark text-sm"></i>
                    </button>
                </div>
            </div>

            // User Session Banner (shown when user is authenticated)
            {move || current_user.get().map(|user| {
                let initial = user.username.chars().next().unwrap_or('U').to_uppercase().to_string();
                let is_staff = user.is_staff;
                let photo = user.photo_url.clone();
                let display_title = user.display_name();
                view! {
                    <div class="mx-3.5 mt-2.5 mb-1 p-2.5 rounded-2xl bg-gradient-to-r from-slate-50 via-slate-50 to-emerald-50/40 border border-slate-200/90 flex items-center justify-between gap-2 shadow-2xs">
                        <button
                            type="button"
                            class="flex items-center space-x-2.5 min-w-0 flex-1 text-left hover:opacity-90 transition cursor-pointer"
                            title="Ver mi perfil cívico, Eco-Puntos e insignias"
                            on:click=move |_| show_user_profile.set(true)
                        >
                            <div class="relative shrink-0">
                                {match photo {
                                    Some(ref url) if !url.trim().is_empty() => view! {
                                        <img
                                            src=url.clone()
                                            class="w-8 h-8 rounded-xl object-cover border border-slate-300 shadow-2xs"
                                            alt="Avatar"
                                        />
                                    }.into_view(),
                                    _ => view! {
                                        <div class=format!(
                                            "w-8 h-8 rounded-xl flex items-center justify-center text-xs font-bold text-white shadow-2xs {}",
                                            if is_staff { "bg-emerald-600" } else { "bg-slate-800" }
                                        )>
                                            {initial}
                                        </div>
                                    }.into_view()
                                }}
                                <span class="absolute -bottom-0.5 -right-0.5 w-2.5 h-2.5 rounded-full bg-emerald-500 ring-2 ring-white"></span>
                            </div>
                            <div class="min-w-0 flex-1">
                                <div class="flex items-center space-x-1.5">
                                    <span class="font-bold text-slate-800 text-xs truncate max-w-[120px] sm:max-w-[140px]">
                                        {display_title}
                                    </span>
                                    {if is_staff {
                                        view! {
                                            <span class="px-1.5 py-0.2 rounded-md bg-purple-50 text-purple-700 border border-purple-200 text-[9px] font-bold uppercase tracking-wider shrink-0">
                                                "Autoridad"
                                            </span>
                                        }.into_view()
                                    } else {
                                        view! {
                                            <span class="px-1.5 py-0.2 rounded-md bg-emerald-50 text-emerald-700 border border-emerald-200 text-[9px] font-bold uppercase tracking-wider shrink-0">
                                                "Ciudadano"
                                            </span>
                                        }.into_view()
                                    }}
                                </div>
                                <p class="text-[10px] text-slate-400 font-medium truncate">
                                    {if is_staff { "Fiscalizador Ambiental" } else { "Participación Activa" }}
                                </p>
                            </div>
                        </button>

                        <div class="flex items-center space-x-1 shrink-0">
                            <button
                                class="px-2 py-1 rounded-xl bg-amber-50 hover:bg-amber-100 text-amber-700 border border-amber-200/80 text-[11px] font-bold flex items-center space-x-1 transition cursor-pointer shadow-2xs"
                                title="Ver mis Eco-Puntos y Medallas"
                                on:click=move |_| show_user_profile.set(true)
                            >
                                <span>"🏆"</span>
                                <span class="text-[10px] font-semibold text-amber-800">"Puntos"</span>
                            </button>
                            <button
                                class="w-7 h-7 rounded-xl text-slate-400 hover:text-rose-600 hover:bg-rose-50 border border-transparent hover:border-rose-100 transition flex items-center justify-center cursor-pointer"
                                title="Cerrar sesión"
                                on:click=move |_| {
                                    crate::storage::logout_and_clear_session(current_user);
                                    show_authority_dashboard.set(false);
                                }
                            >
                                <i class="fa-solid fa-right-from-bracket text-xs"></i>
                            </button>
                        </div>
                    </div>
                }
            })}

            // 2. Google Maps Search Bar
            <div class="px-3.5 pt-2.5 pb-2">
                <div class="relative flex items-center">
                    <i class="fa-solid fa-magnifying-glass text-slate-400 text-xs absolute left-3 pointer-events-none"></i>
                    <input
                        type="text"
                        placeholder="Buscar denuncias, zonas, pasivos..."
                        class="w-full bg-slate-50 hover:bg-slate-100/70 focus:bg-white text-xs text-slate-800 placeholder:text-slate-400 rounded-xl py-2 pl-8 pr-8 border border-slate-200/80 shadow-2xs focus:border-emerald-500 focus:ring-2 focus:ring-emerald-100 focus:outline-none transition-all"
                        prop:value=move || search_query.get()
                        on:input=move |ev| search_query.set(event_target_value(&ev))
                    />
                    {move || (!search_query.get().is_empty()).then(|| view! {
                        <button
                            class="absolute right-2.5 text-slate-400 hover:text-slate-600 p-0.5 rounded hover:bg-slate-200 transition cursor-pointer"
                            title="Limpiar búsqueda"
                            on:click=move |_| search_query.set("".to_string())
                        >
                            <i class="fa-solid fa-xmark text-xs"></i>
                        </button>
                    })}
                </div>
            </div>

            // 3. Quick Action Chips
            <div class="flex items-center space-x-2 px-3.5 pb-2.5 overflow-x-auto no-scrollbar">
                // Report Action Chip
                <button
                    class=move || {
                        if is_report_mode.get() {
                            "bg-rose-600 hover:bg-rose-700 text-white rounded-xl font-semibold px-3 py-1.5 text-xs shadow-xs flex items-center space-x-1.5 shrink-0 transition cursor-pointer"
                        } else {
                            "bg-emerald-600 hover:bg-emerald-700 text-white rounded-xl font-semibold px-3 py-1.5 text-xs shadow-xs flex items-center space-x-1.5 shrink-0 transition cursor-pointer"
                        }
                    }
                    on:click=move |_| {
                        if current_user.get().is_none() {
                            show_auth_dialog.set(true);
                        } else {
                            let curr = is_report_mode.get();
                            let next = !curr;
                            is_report_mode.set(next);
                            if next {
                                is_collapsed.set(true);
                            }
                        }
                    }
                >
                    {move || if is_report_mode.get() {
                        view! {
                            <i class="fa-solid fa-xmark text-xs"></i>
                            <span>"Cancelar"</span>
                        }.into_view()
                    } else {
                        view! {
                            <i class="fa-solid fa-plus text-xs"></i>
                            <span>"Reportar"</span>
                        }.into_view()
                    }}
                </button>

                // Transparency Portal Chip
                <button
                    class=move || {
                        if show_transparency_portal.get() {
                            "bg-emerald-700 text-white rounded-xl font-semibold px-3 py-1.5 text-xs shadow-xs flex items-center space-x-1.5 shrink-0 transition cursor-pointer"
                        } else {
                            "bg-slate-100 hover:bg-emerald-50 text-slate-700 hover:text-emerald-800 rounded-xl border border-slate-200/80 font-semibold px-3 py-1.5 text-xs flex items-center space-x-1.5 shrink-0 transition cursor-pointer"
                        }
                    }
                    on:click=move |_| show_transparency_portal.update(|v| *v = !*v)
                >
                    <span>"🏆"</span>
                    <span>"Transparencia"</span>
                    <span class="px-1.5 py-0.2 rounded-md text-[10px] bg-emerald-100 text-emerald-800 font-bold font-mono">
                        {move || format!("{}", solved_count())}
                    </span>
                </button>

                // Authority Dashboard Chip (Restricted exclusively to authorized staff)
                {move || {
                    let is_authority = current_user.get().map(|u| u.is_authority()).unwrap_or(false);
                    is_authority.then(|| view! {
                        <button
                            class=move || {
                                if show_authority_dashboard.get() {
                                    "bg-slate-900 text-white rounded-xl font-semibold px-3 py-1.5 text-xs shadow-xs flex items-center space-x-1.5 shrink-0 transition cursor-pointer"
                                } else {
                                    "bg-slate-100 hover:bg-slate-200 text-slate-800 rounded-xl border border-slate-200/80 font-semibold px-3 py-1.5 text-xs flex items-center space-x-1.5 shrink-0 transition cursor-pointer"
                                }
                            }
                            on:click=move |_| show_authority_dashboard.update(|v| *v = !*v)
                        >
                            <span>"🏛️"</span>
                            <span>"Fiscalización"</span>
                        </button>
                    })
                }}
            </div>

            // Report Mode Helper Toast (if active)
            {move || is_report_mode.get().then(|| view! {
                <div class="mx-3.5 mb-2 p-2 rounded-xl bg-amber-50 border border-amber-200 text-xs text-amber-800 flex items-center space-x-2 animate-in fade-in">
                    <span class="w-2 h-2 rounded-full bg-amber-500 animate-ping"></span>
                    <span>"Haz clic en el mapa para ubicar el centro de tu reporte."</span>
                </div>
            })}

            // 4. Scrollable Filter & Environmental Insights Body
            <div class="flex-1 overflow-y-auto px-3.5 space-y-4 pb-24 sm:pb-6">
                // KPI Overview Cards
                <div class="grid grid-cols-2 gap-2">
                    <div class="bg-slate-50/90 border border-slate-200/80 rounded-2xl p-3 flex flex-col justify-between shadow-2xs">
                        <div class="flex items-center justify-between">
                            <p class="text-[9.5px] font-bold text-slate-400 uppercase tracking-wider">"Denuncias en Mapa"</p>
                            <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
                        </div>
                        <p class="text-2xl font-black text-slate-900 mt-1 flex items-baseline gap-1">
                            {move || filtered_count()}
                            <span class="text-xs font-semibold text-slate-400">" / " {move || total_count()}</span>
                        </p>
                    </div>
                    <div class="bg-rose-50/70 border border-rose-100 rounded-2xl p-3 flex flex-col justify-between shadow-2xs">
                        <div class="flex items-center justify-between">
                            <p class="text-[9.5px] font-bold text-rose-500 uppercase tracking-wider">"Críticos"</p>
                            <span class="w-2 h-2 rounded-full bg-rose-500 animate-pulse"></span>
                        </div>
                        <p class="text-2xl font-black text-rose-600 mt-1">
                            {move || critical_count()}
                        </p>
                    </div>
                </div>

                // Visual Donut Chart & Category Breakdown (Replicating Flutter's MonthlyDonutChart)
                <div class="bg-slate-50/80 border border-slate-200/80 rounded-2xl p-3 space-y-2.5 shadow-2xs">
                    <div class="flex items-center justify-between text-[10.5px] font-bold text-slate-600 uppercase tracking-wider">
                        <span class="flex items-center gap-1.5">
                            <i class="fa-solid fa-chart-pie text-emerald-600 text-xs"></i>
                            <span>"Distribución de Impacto"</span>
                        </span>
                        <button
                            class="w-6 h-6 rounded-lg text-slate-400 hover:text-slate-700 hover:bg-slate-200/60 flex items-center justify-center transition cursor-pointer"
                            on:click=move |_| show_charts.update(|v| *v = !*v)
                        >
                            <i class=move || if show_charts.get() { "fa-solid fa-chevron-up text-xs" } else { "fa-solid fa-chevron-down text-xs" }></i>
                        </button>
                    </div>

                    {move || show_charts.get().then(|| view! {
                        <div class="flex items-center space-x-3 pt-1">
                            // SVG Donut
                            <div class="relative w-24 h-24 shrink-0">
                                <svg class="w-full h-full transform -rotate-90" viewBox="0 0 42 42">
                                    // Background Circle
                                    <circle cx="21" cy="21" r="15.91549430918954" fill="transparent" stroke="#e2e8f0" stroke-width="6.5"></circle>

                                    // Minería Segment (Amber)
                                    <circle
                                        cx="21" cy="21" r="15.91549430918954" fill="transparent"
                                        stroke="#d97706" stroke-width="6.5"
                                        stroke-dasharray=move || format!("{} {}", mineria_pct(), 100.0 - mineria_pct())
                                        stroke-dashoffset="0"
                                        class="transition-all duration-500"
                                    ></circle>

                                    // Basura Segment (Slate/Green)
                                    <circle
                                        cx="21" cy="21" r="15.91549430918954" fill="transparent"
                                        stroke="#059669" stroke-width="6.5"
                                        stroke-dasharray=move || format!("{} {}", basura_pct(), 100.0 - basura_pct())
                                        stroke-dashoffset=move || format!("-{}", mineria_pct())
                                        class="transition-all duration-500"
                                    ></circle>

                                    // Agua Segment (Sky)
                                    <circle
                                        cx="21" cy="21" r="15.91549430918954" fill="transparent"
                                        stroke="#0284c7" stroke-width="6.5"
                                        stroke-dasharray=move || format!("{} {}", agua_pct(), 100.0 - agua_pct())
                                        stroke-dashoffset=move || format!("-{}", mineria_pct() + basura_pct())
                                        class="transition-all duration-500"
                                    ></circle>

                                    // Aire Segment (Purple)
                                    <circle
                                        cx="21" cy="21" r="15.91549430918954" fill="transparent"
                                        stroke="#7c3aed" stroke-width="6.5"
                                        stroke-dasharray=move || format!("{} {}", aire_pct(), 100.0 - aire_pct())
                                        stroke-dashoffset=move || format!("-{}", mineria_pct() + basura_pct() + agua_pct())
                                        class="transition-all duration-500"
                                    ></circle>
                                </svg>
                                // Center Label
                                <div class="absolute inset-0 flex flex-col items-center justify-center pointer-events-none">
                                    <span class="text-sm font-black text-slate-900 leading-tight">
                                        {move || total_count()}
                                    </span>
                                    <span class="text-[8px] text-slate-400 uppercase font-bold tracking-wider">
                                        "TOTAL"
                                    </span>
                                </div>
                            </div>

                            // Legend List
                            <div class="flex-1 space-y-1.5 text-xs min-w-0">
                                <div class="flex items-center justify-between text-slate-700">
                                    <span class="flex items-center space-x-1.5 truncate">
                                        <span class="w-2 h-2 rounded-full bg-amber-600 shrink-0"></span>
                                        <span class="font-medium text-slate-700">"Minería"</span>
                                    </span>
                                    <span class="font-bold text-slate-900 font-mono text-[11px]">
                                        {move || format!("{} ({:.0}%)", mineria_count(), mineria_pct())}
                                    </span>
                                </div>
                                <div class="flex items-center justify-between text-slate-700">
                                    <span class="flex items-center space-x-1.5 truncate">
                                        <span class="w-2 h-2 rounded-full bg-emerald-600 shrink-0"></span>
                                        <span class="font-medium text-slate-700">"Residuos"</span>
                                    </span>
                                    <span class="font-bold text-slate-900 font-mono text-[11px]">
                                        {move || format!("{} ({:.0}%)", basura_count(), basura_pct())}
                                    </span>
                                </div>
                                <div class="flex items-center justify-between text-slate-700">
                                    <span class="flex items-center space-x-1.5 truncate">
                                        <span class="w-2 h-2 rounded-full bg-sky-600 shrink-0"></span>
                                        <span class="font-medium text-slate-700">"Agua"</span>
                                    </span>
                                    <span class="font-bold text-slate-900 font-mono text-[11px]">
                                        {move || format!("{} ({:.0}%)", agua_count(), agua_pct())}
                                    </span>
                                </div>
                                <div class="flex items-center justify-between text-slate-700">
                                    <span class="flex items-center space-x-1.5 truncate">
                                        <span class="w-2 h-2 rounded-full bg-purple-600 shrink-0"></span>
                                        <span class="font-medium text-slate-700">"Aire"</span>
                                    </span>
                                    <span class="font-bold text-slate-900 font-mono text-[11px]">
                                        {move || format!("{} ({:.0}%)", aire_count(), aire_pct())}
                                    </span>
                                </div>
                            </div>
                        </div>
                    })}
                </div>

                // Severity Distribution Bar
                <div class="bg-slate-50/80 border border-slate-200/80 rounded-2xl p-3 space-y-2 shadow-2xs">
                    <div class="flex items-center justify-between text-[10.5px] font-bold text-slate-600 uppercase tracking-wider">
                        <span>"Distribución de Severidad"</span>
                        <span class="text-slate-400 font-mono text-[10px] font-medium">{move || format!("{} Totales", total_count())}</span>
                    </div>
                    <div class="h-2.5 w-full rounded-full bg-slate-200/80 overflow-hidden flex shadow-inner">
                        <div class="h-full bg-rose-500 transition-all duration-300" style=move || format!("width: {}%", critical_percent()) title="Críticos"></div>
                        <div class="h-full bg-amber-500 transition-all duration-300" style=move || format!("width: {}%", medium_percent()) title="Medios"></div>
                        <div class="h-full bg-emerald-500 transition-all duration-300" style=move || format!("width: {}%", low_percent()) title="Bajos"></div>
                    </div>
                    <div class="grid grid-cols-3 gap-1 pt-0.5 text-[10px] text-slate-600 font-medium">
                        <div class="flex items-center gap-1.5">
                            <span class="w-2 h-2 rounded-full bg-rose-500 shrink-0"></span>
                            <span class="truncate">{move || format!("Crítico: {}", critical_count())}</span>
                        </div>
                        <div class="flex items-center justify-center gap-1.5">
                            <span class="w-2 h-2 rounded-full bg-amber-500 shrink-0"></span>
                            <span class="truncate">{move || format!("Medio: {}", medium_count())}</span>
                        </div>
                        <div class="flex items-center justify-end gap-1.5">
                            <span class="w-2 h-2 rounded-full bg-emerald-500 shrink-0"></span>
                            <span class="truncate">{move || format!("Bajo: {}", low_count())}</span>
                        </div>
                    </div>
                </div>

                // Estado del Reporte (Filtro Replicado de Flutter)
                <div class="space-y-1.5">
                    <span class="text-[10px] font-bold uppercase tracking-wider text-slate-400 block px-0.5">
                        "Estado del Reporte"
                    </span>
                    <div class="bg-slate-100/90 p-1 rounded-xl grid grid-cols-4 gap-1 border border-slate-200/70 text-center">
                        <button
                            class=move || {
                                if status_filter.get() == "todos" {
                                    "py-1.5 px-0.5 rounded-lg text-[10.5px] font-bold bg-white text-slate-900 shadow-2xs transition truncate cursor-pointer"
                                } else {
                                    "py-1.5 px-0.5 rounded-lg text-[10.5px] font-semibold text-slate-500 hover:text-slate-800 transition truncate cursor-pointer"
                                }
                            }
                            on:click=move |_| status_filter.set("todos".to_string())
                        >
                            "Todos"
                        </button>
                        <button
                            class=move || {
                                if status_filter.get() == "pending" {
                                    "py-1.5 px-0.5 rounded-lg text-[10.5px] font-bold bg-white text-amber-700 shadow-2xs transition truncate cursor-pointer"
                                } else {
                                    "py-1.5 px-0.5 rounded-lg text-[10.5px] font-semibold text-slate-500 hover:text-slate-800 transition truncate cursor-pointer"
                                }
                            }
                            on:click=move |_| status_filter.set("pending".to_string())
                        >
                            "Pendientes"
                        </button>
                        <button
                            class=move || {
                                if status_filter.get() == "verified" {
                                    "py-1.5 px-0.5 rounded-lg text-[10.5px] font-bold bg-white text-purple-700 shadow-2xs transition truncate cursor-pointer"
                                } else {
                                    "py-1.5 px-0.5 rounded-lg text-[10.5px] font-semibold text-slate-500 hover:text-slate-800 transition truncate cursor-pointer"
                                }
                            }
                            on:click=move |_| status_filter.set("verified".to_string())
                        >
                            "Inspección"
                        </button>
                        <button
                            class=move || {
                                if status_filter.get() == "solved" {
                                    "py-1.5 px-0.5 rounded-lg text-[10.5px] font-bold bg-white text-emerald-700 shadow-2xs transition truncate cursor-pointer"
                                } else {
                                    "py-1.5 px-0.5 rounded-lg text-[10.5px] font-semibold text-slate-500 hover:text-slate-800 transition truncate cursor-pointer"
                                }
                            }
                            on:click=move |_| status_filter.set("solved".to_string())
                        >
                            "Resueltos"
                        </button>
                    </div>
                </div>

                // Rango Temporal (Filtro Replicado de Flutter)
                <div class="space-y-1.5">
                    <span class="text-[10px] font-bold uppercase tracking-wider text-slate-400 block px-0.5">
                        "Rango Temporal"
                    </span>
                    <div class="bg-slate-100/90 p-1 rounded-xl grid grid-cols-4 gap-1 border border-slate-200/70 text-center">
                        <button
                            class=move || {
                                if days_filter.get() == 0 {
                                    "py-1.5 px-0.5 rounded-lg text-[10.5px] font-bold bg-white text-slate-900 shadow-2xs cursor-pointer"
                                } else {
                                    "py-1.5 px-0.5 rounded-lg text-[10.5px] font-semibold text-slate-500 hover:text-slate-800 transition cursor-pointer"
                                }
                            }
                            on:click=move |_| days_filter.set(0)
                        >
                            "Todos"
                        </button>
                        <button
                            class=move || {
                                if days_filter.get() == 1 {
                                    "py-1.5 px-0.5 rounded-lg text-[10.5px] font-bold bg-white text-slate-900 shadow-2xs cursor-pointer"
                                } else {
                                    "py-1.5 px-0.5 rounded-lg text-[10.5px] font-semibold text-slate-500 hover:text-slate-800 transition cursor-pointer"
                                }
                            }
                            on:click=move |_| days_filter.set(1)
                        >
                            "Hoy"
                        </button>
                        <button
                            class=move || {
                                if days_filter.get() == 7 {
                                    "py-1.5 px-0.5 rounded-lg text-[10.5px] font-bold bg-white text-slate-900 shadow-2xs cursor-pointer"
                                } else {
                                    "py-1.5 px-0.5 rounded-lg text-[10.5px] font-semibold text-slate-500 hover:text-slate-800 transition cursor-pointer"
                                }
                            }
                            on:click=move |_| days_filter.set(7)
                        >
                            "7 días"
                        </button>
                        <button
                            class=move || {
                                if days_filter.get() == 30 {
                                    "py-1.5 px-0.5 rounded-lg text-[10.5px] font-bold bg-white text-slate-900 shadow-2xs cursor-pointer"
                                } else {
                                    "py-1.5 px-0.5 rounded-lg text-[10.5px] font-semibold text-slate-500 hover:text-slate-800 transition cursor-pointer"
                                }
                            }
                            on:click=move |_| days_filter.set(30)
                        >
                            "30 días"
                        </button>
                    </div>
                </div>

                // Category Filter Chips
                <div class="space-y-1.5">
                    <span class="text-[10px] font-bold uppercase tracking-wider text-slate-400 block px-0.5">
                        "Categoría de Impacto"
                    </span>
                    <div class="grid grid-cols-2 gap-2">
                        {EcoCategory::all().iter().map(|&cat| {
                            let is_selected = move || selected_categories.get().contains(&cat);
                            view! {
                                <button
                                    class=move || {
                                        if is_selected() {
                                            format!(
                                                "py-2.5 px-3 rounded-2xl text-xs font-bold flex items-center space-x-2 border transition shadow-2xs cursor-pointer {}",
                                                match cat {
                                                    EcoCategory::Mineria => "bg-amber-50/90 text-amber-900 border-amber-300 ring-1 ring-amber-300/50",
                                                    EcoCategory::Basura => "bg-emerald-50/90 text-emerald-900 border-emerald-300 ring-1 ring-emerald-300/50",
                                                    EcoCategory::Agua => "bg-sky-50/90 text-sky-900 border-sky-300 ring-1 ring-sky-300/50",
                                                    EcoCategory::Aire => "bg-purple-50/90 text-purple-900 border-purple-300 ring-1 ring-purple-300/50",
                                                }
                                            )
                                        } else {
                                            "py-2.5 px-3 rounded-2xl text-xs font-semibold flex items-center space-x-2 bg-slate-50 text-slate-600 border border-slate-200/70 hover:bg-slate-100 hover:border-slate-300 transition cursor-pointer".to_string()
                                        }
                                    }
                                    on:click=move |_| toggle_category(cat)
                                >
                                    <span class="text-base shrink-0">{cat.icon()}</span>
                                    <span class="truncate">{cat.display_name()}</span>
                                </button>
                            }
                        }).collect_view()}
                    </div>
                </div>

                // Severity Filter (Segmented Button)
                <div class="space-y-1.5">
                    <span class="text-[10px] font-bold uppercase tracking-wider text-slate-400 block px-0.5">
                        "Nivel de Severidad"
                    </span>
                    <div class="bg-slate-100/90 p-1 rounded-xl flex space-x-1 border border-slate-200/70">
                        {EcoSeverity::all().iter().map(|&sev| {
                            let is_selected = move || selected_severities.get().contains(&sev);
                            view! {
                                <button
                                    class=move || {
                                        if is_selected() {
                                            match sev {
                                                EcoSeverity::Critico => "flex-1 py-1.5 rounded-lg text-xs font-bold bg-white text-rose-600 shadow-2xs border border-rose-100 transition cursor-pointer",
                                                EcoSeverity::Medio => "flex-1 py-1.5 rounded-lg text-xs font-bold bg-white text-amber-600 shadow-2xs border border-amber-100 transition cursor-pointer",
                                                EcoSeverity::Bajo => "flex-1 py-1.5 rounded-lg text-xs font-bold bg-white text-emerald-600 shadow-2xs border border-emerald-100 transition cursor-pointer",
                                            }
                                        } else {
                                            "flex-1 py-1.5 rounded-lg text-xs font-semibold text-slate-400 hover:text-slate-700 transition cursor-pointer"
                                        }
                                    }
                                    on:click=move |_| select_severity(sev)
                                >
                                    {match sev {
                                        EcoSeverity::Critico => "Crítico",
                                        EcoSeverity::Medio => "Medio",
                                        EcoSeverity::Bajo => "Bajo",
                                    }}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                </div>

                // District Filter (Google Chips)
                <div class="space-y-1.5">
                    <span class="text-[10px] font-bold uppercase tracking-wider text-slate-400 block px-0.5">
                        "Distrito Municipal"
                    </span>
                    <div class="flex flex-wrap gap-1.5">
                        <button
                            class=move || {
                                if selected_district.get().is_none() {
                                    "px-3 py-1.5 rounded-xl text-xs font-bold bg-slate-900 text-white shadow-2xs cursor-pointer"
                                } else {
                                    "px-3 py-1.5 rounded-xl text-xs font-semibold bg-slate-100 text-slate-600 hover:bg-slate-200/80 border border-slate-200/70 transition cursor-pointer"
                                }
                            }
                            on:click=move |_| selected_district.set(None)
                        >
                            "Todos"
                        </button>
                        {EcoDistrict::all().iter().map(|&dist| {
                            let is_selected = move || selected_district.get() == Some(dist);
                            view! {
                                <button
                                    class=move || {
                                        if is_selected() {
                                            "px-3 py-1.5 rounded-xl text-xs font-bold bg-slate-900 text-white shadow-2xs cursor-pointer"
                                        } else {
                                            "px-3 py-1.5 rounded-xl text-xs font-semibold bg-slate-100 text-slate-600 hover:bg-slate-200/80 border border-slate-200/70 transition cursor-pointer"
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

            </div>

            // 5. Production Live Sync Status Footer
            <div class="px-4 py-2.5 pb-safe border-t border-slate-100 bg-slate-50/90 text-[10.5px] text-slate-500 flex items-center justify-between shrink-0">
                <span class="font-semibold text-slate-600">"EcoAlerta · Pasco"</span>
                <button
                    type="button"
                    class="flex items-center space-x-1.5 font-medium transition cursor-pointer hover:opacity-80"
                    title="Estado de sincronización con el servidor municipal (clic para sincronizar ahora)"
                    on:click=move |_| on_manual_sync.call(())
                >
                    {move || match sync_status.get() {
                        SyncStatus::RealtimeWs => view! {
                            <span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
                            <span class="text-emerald-700 font-semibold">"Tiempo real · WebSocket"</span>
                        }.into_view(),
                        SyncStatus::FallbackHttp => view! {
                            <span class="w-2 h-2 rounded-full bg-teal-500"></span>
                            <span class="text-teal-700 font-medium">"Sincronizado · Respaldo HTTP"</span>
                        }.into_view(),
                        SyncStatus::Connecting => view! {
                            <span class="w-2 h-2 rounded-full border-2 border-sky-500 border-t-transparent animate-spin"></span>
                            <span class="text-sky-700 font-medium">"Conectando tiempo real..."</span>
                        }.into_view(),
                        SyncStatus::Offline => view! {
                            <span class="w-2 h-2 rounded-full bg-rose-500"></span>
                            <span class="text-rose-600 underline decoration-rose-300 font-medium">"Reconectando · Reintentar"</span>
                        }.into_view(),
                    }}
                </button>
            </div>
        </aside>
    }
}
