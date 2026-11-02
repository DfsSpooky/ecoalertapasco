use leptos::*;
use crate::api::profile_url;
use crate::models::{AuthUser, UserMedal, UserProfile};

struct AwardSpec {
    id: &'static str,
    title: &'static str,
    requirement: &'static str,
    active_gradient: &'static str,
    active_shadow: &'static str,
}

const OFFICIAL_AWARDS: [AwardSpec; 5] = [
    AwardSpec {
        id: "first_report",
        title: "Primer Paso",
        requirement: "1 reporte",
        active_gradient: "from-[#34C759] via-[#10B981] to-[#047857]",
        active_shadow: "shadow-[0_8px_18px_-4px_rgba(16,185,129,0.45)]",
    },
    AwardSpec {
        id: "first_solved",
        title: "Impacto Real",
        requirement: "1 resuelto",
        active_gradient: "from-[#30D158] via-[#059669] to-[#065F46]",
        active_shadow: "shadow-[0_8px_18px_-4px_rgba(5,150,105,0.45)]",
    },
    AwardSpec {
        id: "vigilant",
        title: "Centinela",
        requirement: "5 reportes",
        active_gradient: "from-[#0A84FF] via-[#007AFF] to-[#0055D4]",
        active_shadow: "shadow-[0_8px_18px_-4px_rgba(0,122,255,0.42)]",
    },
    AwardSpec {
        id: "community",
        title: "Voz Vecinal",
        requirement: "3 respaldos",
        active_gradient: "from-[#7D7AFF] via-[#5856D6] to-[#3634A3]",
        active_shadow: "shadow-[0_8px_18px_-4px_rgba(88,86,214,0.42)]",
    },
    AwardSpec {
        id: "master_eco",
        title: "Héroe Pasco",
        requirement: "3 resueltos",
        active_gradient: "from-[#FFD60A] via-[#FF9F0A] to-[#D97706]",
        active_shadow: "shadow-[0_8px_18px_-4px_rgba(245,158,11,0.45)]",
    },
];

fn render_award_svg(id: &str) -> View {
    match id {
        "first_report" => view! {
            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M11 20A7 7 0 0 1 9.8 6.1C15.5 5 17 4.48 19 2c1 2 2 4.18 2 8 0 5.5-4.78 10-10 10Z"/>
                <path d="M2 21c0-3 1.85-5.36 5.08-6C9.5 14.52 12 13 13 12"/>
            </svg>
        }.into_view(),
        "first_solved" => view! {
            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
                <path d="m9 12 2 2 4-4"/>
            </svg>
        }.into_view(),
        "vigilant" => view! {
            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0"/>
                <circle cx="12" cy="12" r="3"/>
            </svg>
        }.into_view(),
        "community" => view! {
            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/>
                <circle cx="9" cy="7" r="4"/>
                <path d="M22 21v-2a4 4 0 0 0-3-3.87"/>
                <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
            </svg>
        }.into_view(),
        _ => view! {
            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
            </svg>
        }.into_view(),
    }
}

fn render_rank_emblem_svg(level: u32) -> View {
    if level >= 3 {
        view! {
            <svg class="w-7 h-7 text-white drop-shadow-sm" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
                <polygon points="12 7.5 13.4 10.4 16.5 10.8 14.2 13 14.8 16.1 12 14.6 9.2 16.1 9.8 13 7.5 10.8 10.6 10.4 12 7.5"/>
            </svg>
        }.into_view()
    } else {
        view! {
            <svg class="w-7 h-7 text-white drop-shadow-sm" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round">
                <path d="M11 20A7 7 0 0 1 9.8 6.1C15.5 5 17 4.48 19 2c1 2 2 4.18 2 8 0 5.5-4.78 10-10 10Z"/>
                <path d="M2 21c0-3 1.85-5.36 5.08-6C9.5 14.52 12 13 13 12"/>
            </svg>
        }.into_view()
    }
}

#[component]
pub fn UserProfileModal(
    show_modal: RwSignal<bool>,
    current_user: RwSignal<Option<AuthUser>>,
) -> impl IntoView {
    let dialog_ref = create_node_ref::<leptos::html::Dialog>();
    let pressed_outside = create_rw_signal(false);

    let profile = create_rw_signal(None::<UserProfile>);
    let is_loading = create_rw_signal(false);
    let error_msg = create_rw_signal(None::<String>);

    // Sync reactive signal with native <dialog> Top Layer modal API
    create_effect(move |_| {
        let should_open = show_modal.get();
        if let Some(dialog_el) = dialog_ref.get() {
            if should_open {
                if !dialog_el.open() {
                    let _ = dialog_el.show_modal();
                }
            } else if dialog_el.open() {
                dialog_el.close();
            }
        }
    });

    // Fetch live profile stats whenever modal opens
    create_effect(move |_| {
        if show_modal.get() {
            if let Some(user) = current_user.get() {
                is_loading.set(true);
                error_msg.set(None);
                let token = user.token.clone();

                spawn_local(async move {
                    let req = gloo_net::http::Request::get(&profile_url())
                        .header("Authorization", &format!("Token {}", token))
                        .send()
                        .await;

                    match req {
                        Ok(resp) => {
                            if resp.ok() {
                                if let Ok(data) = resp.json::<UserProfile>().await {
                                    profile.set(Some(data));
                                } else {
                                    error_msg.set(Some("No se pudieron interpretar los datos del perfil.".to_string()));
                                }
                            } else {
                                error_msg.set(Some(format!("No se pudo cargar el perfil ({})", resp.status())));
                            }
                        }
                        Err(e) => {
                            error_msg.set(Some(format!("Error de conexión: {}", e)));
                        }
                    }
                    is_loading.set(false);
                });
            }
        }
    });

    let is_pointer_outside = |ev: &web_sys::PointerEvent, el: &web_sys::HtmlDialogElement| -> bool {
        let r = el.get_bounding_client_rect();
        let x = ev.client_x() as f64;
        let y = ev.client_y() as f64;
        x < r.left() || x > r.right() || y < r.top() || y > r.bottom()
    };

    view! {
        <dialog
            node_ref=dialog_ref
            class="apple-modal-dialog select-none"
            aria-labelledby="user-profile-modal-title"
            attr:closedby="any"
            on:close=move |_| {
                pressed_outside.set(false);
                show_modal.set(false);
            }
            on:pointerdown=move |ev| {
                if let Some(el) = dialog_ref.get() {
                    pressed_outside.set(is_pointer_outside(&ev, &el));
                }
            }
            on:pointerup=move |ev| {
                if let Some(el) = dialog_ref.get() {
                    if pressed_outside.get_untracked() && is_pointer_outside(&ev, &el) {
                        el.close();
                    }
                    pressed_outside.set(false);
                }
            }
        >
            // Mobile iOS Sheet Grabber
            <div class="pt-2.5 pb-0.5 flex justify-center sm:hidden bg-[#F5F5F7]/90 shrink-0">
                <div class="w-9 h-1 rounded-full bg-black/15"></div>
            </div>

            // Apple Translucent Sheet Navigation Bar
            <header class="px-6 pt-3.5 sm:pt-5 pb-3.5 bg-[#F5F5F7]/88 backdrop-blur-xl border-b border-black/[0.06] flex items-center justify-between gap-4 shrink-0">
                <div class="min-w-0">
                    <h2
                        id="user-profile-modal-title"
                        class="text-[17px] font-semibold tracking-[-0.022em] text-[#1D1D1F] leading-tight"
                    >
                        "Perfil Cívico"
                    </h2>
                    <p class="text-[12px] text-[#86868B] leading-snug mt-0.5 truncate">
                        "Impacto ciudadano y Eco-Puntos · Cerro de Pasco"
                    </p>
                </div>

                <button
                    type="button"
                    autofocus
                    aria-label="Cerrar perfil ciudadano"
                    class="w-7 h-7 rounded-full bg-black/[0.06] hover:bg-black/[0.11] active:scale-95 text-[#3C3C43]/75 hover:text-[#1D1D1F] flex items-center justify-center transition-all shrink-0 cursor-pointer focus:outline-none focus-visible:ring-2 focus-visible:ring-[#007AFF]/50"
                    on:click=move |_| show_modal.set(false)
                >
                    <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round">
                        <line x1="18" y1="6" x2="6" y2="18"></line>
                        <line x1="6" y1="6" x2="18" y2="18"></line>
                    </svg>
                </button>
            </header>

            // Scrollable Grouped Sheet Body
            <div class="px-5 sm:px-6 py-5 overflow-y-auto overscroll-contain space-y-5 text-[#1D1D1F] flex-1 min-h-0">
                {move || {
                    if is_loading.get() && profile.get().is_none() {
                        view! {
                            <div class="py-16 flex flex-col items-center justify-center space-y-3">
                                <div class="w-6 h-6 border-2 border-black/15 border-t-[#1D1D1F] rounded-full animate-spin"></div>
                                <p class="text-[12.5px] font-medium text-[#86868B]">
                                    "Actualizando actividad cívica…"
                                </p>
                            </div>
                        }.into_view()
                    } else if let Some(err) = error_msg.get() {
                        view! {
                            <div class="p-4 bg-white rounded-[18px] shadow-[0_2px_10px_rgba(0,0,0,0.03),0_0_0_0.5px_rgba(255,59,48,0.28)] flex items-start gap-3">
                                <div class="w-7 h-7 rounded-full bg-[#FF3B30]/10 text-[#D70015] flex items-center justify-center shrink-0 mt-0.5">
                                    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round">
                                        <circle cx="12" cy="12" r="10"/>
                                        <line x1="12" y1="8" x2="12" y2="12"/>
                                        <line x1="12" y1="16" x2="12.01" y2="16"/>
                                    </svg>
                                </div>
                                <div class="text-[12.5px] text-[#1D1D1F] leading-relaxed">
                                    <p class="font-semibold text-[#D70015]">"No se pudo sincronizar el perfil"</p>
                                    <p class="text-[#6E6E73] mt-0.5">{err}</p>
                                </div>
                            </div>
                        }.into_view()
                    } else if let Some(prof) = profile.get() {
                        let pct = if prof.next_level_points > 0 {
                            ((prof.eco_points as f64 / prof.next_level_points as f64) * 100.0).clamp(0.0, 100.0)
                        } else {
                            100.0
                        };
                        let remaining_pts = prof.next_level_points.saturating_sub(prof.eco_points);

                        // Apple Activity Ring geometry (r = 19, circumference = 119.38)
                        let ring_circumference = 119.38_f64;
                        let visual_pct = if prof.eco_points == 0 { 3.5 } else { pct.max(5.0) };
                        let dash_offset = ring_circumference * (1.0 - visual_pct / 100.0);

                        let user_photo = current_user
                            .get()
                            .and_then(|u| u.photo_url)
                            .filter(|u| !u.trim().is_empty());

                        let display_name = current_user
                            .get()
                            .map(|u| u.display_name())
                            .filter(|n| !n.trim().is_empty())
                            .unwrap_or_else(|| prof.username.clone());

                        let email_label = if prof.email.trim().is_empty() {
                            "Cuenta verificada en EcoAlerta".to_string()
                        } else {
                            prof.email.clone()
                        };

                        let unlocked_medals: Vec<UserMedal> = prof.medals.clone();
                        let unlocked_count = unlocked_medals.len();

                        view! {
                            // 1. Hero Identity & Activity Ring Card (Apple Account + Fitness Style)
                            <section class="bg-white rounded-[22px] p-5 shadow-[0_2px_12px_rgba(0,0,0,0.035),0_0_0_0.5px_rgba(0,0,0,0.06)]">
                                <div class="flex items-center justify-between gap-4">
                                    <div class="flex items-center gap-3.5 min-w-0">
                                        {if let Some(photo_src) = user_photo {
                                            view! {
                                                <div class="relative shrink-0">
                                                    <img
                                                        src=photo_src
                                                        alt=""
                                                        class="w-14 h-14 rounded-[18px] object-cover ring-1 ring-black/[0.08] shadow-sm"
                                                    />
                                                    <span class="absolute -bottom-1 -right-1 w-5 h-5 rounded-full bg-[#34C759] ring-2 ring-white flex items-center justify-center text-white">
                                                        <svg class="w-2.5 h-2.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                                                            <polyline points="20 6 9 17 4 12"></polyline>
                                                        </svg>
                                                    </span>
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <div class="w-14 h-14 rounded-[18px] bg-gradient-to-b from-[#166534] via-[#065F46] to-[#022C22] shadow-[0_8px_18px_-4px_rgba(6,78,59,0.34),inset_0_1px_0_rgba(255,255,255,0.26)] flex items-center justify-center shrink-0">
                                                    {render_rank_emblem_svg(prof.level)}
                                                </div>
                                            }.into_view()
                                        }}

                                        <div class="min-w-0">
                                            <h3 class="text-[17px] font-semibold tracking-[-0.022em] text-[#1D1D1F] leading-snug truncate">
                                                {display_name}
                                            </h3>
                                            <p class="text-[12px] text-[#86868B] truncate">
                                                {email_label}
                                            </p>
                                            <div class="flex flex-wrap items-center gap-1.5 mt-1.5">
                                                <span class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-[#34C759]/[0.12] text-[#00704E] text-[11px] font-semibold tracking-[-0.01em]">
                                                    <span class="w-1.5 h-1.5 rounded-full bg-[#34C759]"></span>
                                                    {prof.rank.clone()}
                                                </span>
                                                {prof.is_staff.then(|| view! {
                                                    <span class="inline-flex items-center px-2.5 py-0.5 rounded-full bg-[#5856D6]/[0.10] text-[#4240B3] text-[11px] font-semibold tracking-[-0.01em]">
                                                        "Personal Autorizado"
                                                    </span>
                                                })}
                                            </div>
                                        </div>
                                    </div>

                                    // Right Score + Apple Activity Ring
                                    <div class="flex items-center gap-3 shrink-0">
                                        <div class="text-right">
                                            <span class="block text-[28px] sm:text-[32px] font-bold tracking-[-0.035em] tabular-nums text-[#1D1D1F] leading-none">
                                                {prof.eco_points}
                                            </span>
                                            <span class="block text-[11px] font-medium text-[#86868B] mt-1">
                                                "Eco-Puntos"
                                            </span>
                                        </div>

                                        <div class="relative w-12 h-12 hidden sm:flex items-center justify-center shrink-0" aria-hidden="true">
                                            <svg class="w-12 h-12 -rotate-90" viewBox="0 0 48 48">
                                                <defs>
                                                    <linearGradient id="appleEcoRingGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                                                        <stop offset="0%" stop-color="#34C759" />
                                                        <stop offset="100%" stop-color="#00825A" />
                                                    </linearGradient>
                                                </defs>
                                                <circle
                                                    cx="24"
                                                    cy="24"
                                                    r="19"
                                                    fill="none"
                                                    stroke="rgba(0,0,0,0.06)"
                                                    stroke-width="4.5"
                                                />
                                                <circle
                                                    cx="24"
                                                    cy="24"
                                                    r="19"
                                                    fill="none"
                                                    stroke="url(#appleEcoRingGrad)"
                                                    stroke-width="4.5"
                                                    stroke-linecap="round"
                                                    stroke-dasharray=format!("{:.2}", ring_circumference)
                                                    stroke-dashoffset=format!("{:.2}", dash_offset)
                                                    class="transition-all duration-700 ease-out"
                                                />
                                            </svg>
                                            <span class="absolute text-[10px] font-bold tabular-nums text-[#1D1D1F] tracking-tight">
                                                {format!("Nv.{}", prof.level)}
                                            </span>
                                        </div>
                                    </div>
                                </div>

                                // Level Progress Bar (Hairline Separated)
                                <div class="mt-4 pt-3.5 border-t border-black/[0.06] space-y-2">
                                    <div class="flex items-center justify-between text-[12px]">
                                        <span class="font-medium text-[#1D1D1F] flex items-center gap-1.5">
                                            "Nivel Cívico"
                                            <span class="inline-flex items-center justify-center px-1.5 py-0.5 rounded-md bg-black/[0.05] text-[#1D1D1F] font-semibold tabular-nums text-[11px]">
                                                {prof.level}
                                            </span>
                                        </span>
                                        <span class="text-[#86868B] tabular-nums">
                                            {if remaining_pts > 0 {
                                                format!("{} / {} pts · faltan {}", prof.eco_points, prof.next_level_points, remaining_pts)
                                            } else {
                                                format!("{} pts · Rango máximo", prof.eco_points)
                                            }}
                                        </span>
                                    </div>
                                    <div class="w-full h-1.5 bg-black/[0.06] rounded-full overflow-hidden">
                                        <div
                                            class="h-full bg-gradient-to-r from-[#34C759] to-[#00825A] rounded-full transition-all duration-700 ease-out"
                                            style=format!("width: {:.1}%;", if prof.eco_points == 0 { 0.0 } else { pct.max(4.0) })
                                        ></div>
                                    </div>
                                </div>
                            </section>

                            // 2. Unified 4-Stat Apple Bento Strip (Hairline Divided, Zero Rainbow Boxes)
                            <section
                                aria-label="Resumen de contribuciones"
                                class="bg-white rounded-[20px] shadow-[0_2px_12px_rgba(0,0,0,0.035),0_0_0_0.5px_rgba(0,0,0,0.06)] grid grid-cols-2 sm:grid-cols-4 overflow-hidden"
                            >
                                <div class="p-3.5 sm:p-4 border-b sm:border-b-0 border-r border-black/[0.06]">
                                    <div class="flex items-center gap-1.5 text-[#86868B]">
                                        <svg class="w-3.5 h-3.5 text-[#FF9500] shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                                            <path d="M4 15s1-1 4-1 5 2 8 2 4-1 4-1V3s-1 1-4 1-5-2-8-2-4 1-4 1z"/>
                                            <line x1="4" y1="22" x2="4" y2="15"/>
                                        </svg>
                                        <span class="text-[11.5px] font-medium truncate">"Reportadas"</span>
                                    </div>
                                    <span class="block text-[23px] font-semibold tracking-[-0.03em] tabular-nums text-[#1D1D1F] leading-tight mt-1.5">
                                        {prof.total_reported}
                                    </span>
                                </div>

                                <div class="p-3.5 sm:p-4 border-b sm:border-b-0 sm:border-r border-black/[0.06]">
                                    <div class="flex items-center gap-1.5 text-[#86868B]">
                                        <svg class="w-3.5 h-3.5 text-[#34C759] shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                                            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
                                            <polyline points="22 4 12 14.01 9 11.01"/>
                                        </svg>
                                        <span class="text-[11.5px] font-medium truncate">"Solucionadas"</span>
                                    </div>
                                    <span class="block text-[23px] font-semibold tracking-[-0.03em] tabular-nums text-[#1D1D1F] leading-tight mt-1.5">
                                        {prof.total_solved}
                                    </span>
                                </div>

                                <div class="p-3.5 sm:p-4 border-r border-black/[0.06]">
                                    <div class="flex items-center gap-1.5 text-[#86868B]">
                                        <svg class="w-3.5 h-3.5 text-[#007AFF] shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                                            <circle cx="12" cy="12" r="10"/>
                                            <polyline points="12 6 12 12 16 14"/>
                                        </svg>
                                        <span class="text-[11.5px] font-medium truncate">"En inspección"</span>
                                    </div>
                                    <span class="block text-[23px] font-semibold tracking-[-0.03em] tabular-nums text-[#1D1D1F] leading-tight mt-1.5">
                                        {prof.total_in_inspection}
                                    </span>
                                </div>

                                <div class="p-3.5 sm:p-4">
                                    <div class="flex items-center gap-1.5 text-[#86868B]">
                                        <svg class="w-3.5 h-3.5 text-[#5856D6] shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                                            <path d="M7 10v12"/>
                                            <path d="M15 5.88 14 10h5.83a2 2 0 0 1 1.92 2.56l-2.33 8A2 2 0 0 1 17.5 22H4a2 2 0 0 1-2-2v-8a2 2 0 0 1 2-2h2.76a2 2 0 0 0 1.79-1.11L12 2a3.13 3.13 0 0 1 3 3.88Z"/>
                                        </svg>
                                        <span class="text-[11.5px] font-medium truncate">"Respaldos"</span>
                                    </div>
                                    <span class="block text-[23px] font-semibold tracking-[-0.03em] tabular-nums text-[#1D1D1F] leading-tight mt-1.5">
                                        {prof.total_confirmations_given}
                                    </span>
                                </div>
                            </section>

                            // 3. Apple Fitness Civic Awards Showcase
                            <section class="space-y-2.5">
                                <div class="px-1 flex items-center justify-between">
                                    <h4 class="text-[13px] font-semibold tracking-[-0.01em] text-[#1D1D1F]">
                                        "Insignias cívicas"
                                    </h4>
                                    <span class="text-[12px] text-[#86868B] tabular-nums">
                                        {format!("{} de {} obtenidas", unlocked_count, OFFICIAL_AWARDS.len().max(unlocked_count))}
                                    </span>
                                </div>

                                <div class="bg-white rounded-[20px] p-4 shadow-[0_2px_12px_rgba(0,0,0,0.035),0_0_0_0.5px_rgba(0,0,0,0.06)]">
                                    <div class="grid grid-cols-3 sm:grid-cols-5 gap-3">
                                        {OFFICIAL_AWARDS.iter().map(|spec| {
                                            let unlocked_opt = unlocked_medals.iter().find(|m| m.id == spec.id);
                                            let is_unlocked = unlocked_opt.is_some();
                                            let subtitle = if is_unlocked { "Obtenida" } else { spec.requirement };

                                            view! {
                                                <div class="flex flex-col items-center text-center group">
                                                    <div class={if is_unlocked {
                                                        format!(
                                                            "relative w-12 h-12 rounded-full bg-gradient-to-b {} {} text-white flex items-center justify-center ring-2 ring-white transition-transform duration-200 group-hover:scale-105",
                                                            spec.active_gradient,
                                                            spec.active_shadow
                                                        )
                                                    } else {
                                                        "relative w-12 h-12 rounded-full bg-[#F5F5F7] text-[#AEAEB2] ring-1 ring-inset ring-black/[0.06] flex items-center justify-center transition-colors".to_string()
                                                    }}>
                                                        {render_award_svg(spec.id)}
                                                        {is_unlocked.then(|| view! {
                                                            <span class="absolute -bottom-0.5 -right-0.5 w-4 h-4 rounded-full bg-[#34C759] ring-2 ring-white text-white flex items-center justify-center">
                                                                <svg class="w-2.5 h-2.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3.2" stroke-linecap="round" stroke-linejoin="round">
                                                                    <polyline points="20 6 9 17 4 12"></polyline>
                                                                </svg>
                                                            </span>
                                                        })}
                                                    </div>

                                                    <span class={if is_unlocked {
                                                        "text-[11px] font-semibold text-[#1D1D1F] leading-tight mt-2"
                                                    } else {
                                                        "text-[11px] font-medium text-[#86868B] leading-tight mt-2"
                                                    }}>
                                                        {spec.title}
                                                    </span>
                                                    <span class={if is_unlocked {
                                                        "text-[10px] font-semibold text-[#00825A] leading-tight mt-0.5"
                                                    } else {
                                                        "text-[10px] text-[#AEAEB2] leading-tight mt-0.5"
                                                    }}>
                                                        {subtitle}
                                                    </span>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>

                                    {if unlocked_medals.is_empty() {
                                        view! {
                                            <p class="mt-3.5 pt-3 border-t border-black/[0.05] text-center text-[11.5px] text-[#86868B] leading-relaxed">
                                                "Envía un reporte ambiental o respalda casos de tu barrio para desbloquear tu primera insignia."
                                            </p>
                                        }.into_view()
                                    } else {
                                        view! {
                                            <div class="mt-3.5 pt-3 border-t border-black/[0.05] divide-y divide-black/[0.05]">
                                                {unlocked_medals.into_iter().map(|m| {
                                                    let id_copy = m.id.clone();
                                                    view! {
                                                        <div class="py-2 first:pt-0 last:pb-0 flex items-center justify-between gap-3">
                                                            <div class="flex items-center gap-2.5 min-w-0">
                                                                <span class="w-6 h-6 rounded-full bg-[#34C759]/12 text-[#00704E] flex items-center justify-center shrink-0">
                                                                    {render_award_svg(&id_copy)}
                                                                </span>
                                                                <div class="min-w-0">
                                                                    <p class="text-[12px] font-semibold text-[#1D1D1F] truncate">{m.title}</p>
                                                                    <p class="text-[11px] text-[#86868B] truncate">{m.desc}</p>
                                                                </div>
                                                            </div>
                                                            <span class="text-[10.5px] font-semibold text-[#00825A] bg-[#34C759]/12 px-2 py-0.5 rounded-full shrink-0">
                                                                "Activa"
                                                            </span>
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        }.into_view()
                                    }}
                                </div>
                            </section>

                            // 4. iOS Settings Inset Grouped List: How to Earn Eco-Points
                            <section class="space-y-2">
                                <h4 class="px-1 text-[13px] font-semibold tracking-[-0.01em] text-[#1D1D1F]">
                                    "Cómo sumar Eco-Puntos"
                                </h4>

                                <ul class="bg-white rounded-[20px] shadow-[0_2px_12px_rgba(0,0,0,0.035),0_0_0_0.5px_rgba(0,0,0,0.06)] divide-y divide-black/[0.06] overflow-hidden">
                                    <li class="px-4 py-3 flex items-center justify-between gap-3">
                                        <div class="flex items-center gap-3 min-w-0">
                                            <span class="w-7 h-7 rounded-[8px] bg-[#007AFF]/12 text-[#007AFF] flex items-center justify-center shrink-0">
                                                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                                                    <path d="M14.5 4h-5L7 7H4a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-3l-2.5-3z"/>
                                                    <circle cx="12" cy="13" r="3"/>
                                                </svg>
                                            </span>
                                            <span class="text-[12.5px] text-[#1D1D1F] leading-snug">
                                                "Reportar incidente con evidencia fotográfica"
                                            </span>
                                        </div>
                                        <span class="px-2.5 py-0.5 rounded-full bg-[#34C759]/12 text-[#00704E] text-[11.5px] font-semibold tabular-nums shrink-0">
                                            "+50 pts"
                                        </span>
                                    </li>

                                    <li class="px-4 py-3 flex items-center justify-between gap-3">
                                        <div class="flex items-center gap-3 min-w-0">
                                            <span class="w-7 h-7 rounded-[8px] bg-[#34C759]/14 text-[#00825A] flex items-center justify-center shrink-0">
                                                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                                                    <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
                                                    <path d="m9 12 2 2 4-4"/>
                                                </svg>
                                            </span>
                                            <span class="text-[12.5px] text-[#1D1D1F] leading-snug">
                                                "Solución verificada por cuadrilla municipal"
                                            </span>
                                        </div>
                                        <span class="px-2.5 py-0.5 rounded-full bg-[#34C759]/12 text-[#00704E] text-[11.5px] font-semibold tabular-nums shrink-0">
                                            "+100 pts"
                                        </span>
                                    </li>

                                    <li class="px-4 py-3 flex items-center justify-between gap-3">
                                        <div class="flex items-center gap-3 min-w-0">
                                            <span class="w-7 h-7 rounded-[8px] bg-[#5856D6]/12 text-[#5856D6] flex items-center justify-center shrink-0">
                                                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                                                    <path d="M7 10v12"/>
                                                    <path d="M15 5.88 14 10h5.83a2 2 0 0 1 1.92 2.56l-2.33 8A2 2 0 0 1 17.5 22H4a2 2 0 0 1-2-2v-8a2 2 0 0 1 2-2h2.76a2 2 0 0 0 1.79-1.11L12 2a3.13 3.13 0 0 1 3 3.88Z"/>
                                                </svg>
                                            </span>
                                            <span class="text-[12.5px] text-[#1D1D1F] leading-snug">
                                                "Respaldar y validar denuncia de otro vecino"
                                            </span>
                                        </div>
                                        <span class="px-2.5 py-0.5 rounded-full bg-[#34C759]/12 text-[#00704E] text-[11.5px] font-semibold tabular-nums shrink-0">
                                            "+10 pts"
                                        </span>
                                    </li>

                                    <li class="px-4 py-3 flex items-center justify-between gap-3">
                                        <div class="flex items-center gap-3 min-w-0">
                                            <span class="w-7 h-7 rounded-[8px] bg-[#30B0C7]/14 text-[#007B94] flex items-center justify-center shrink-0">
                                                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                                                    <path d="M3 12a9 9 0 0 1 15-6.7L21 8"/>
                                                    <path d="M21 3v5h-5"/>
                                                    <path d="M21 12a9 9 0 0 1-15 6.7L3 16"/>
                                                    <path d="M3 21v-5h5"/>
                                                </svg>
                                            </span>
                                            <span class="text-[12.5px] text-[#1D1D1F] leading-snug">
                                                "Actualizar llenado de contenedor de acopio"
                                            </span>
                                        </div>
                                        <span class="px-2.5 py-0.5 rounded-full bg-[#34C759]/12 text-[#00704E] text-[11.5px] font-semibold tabular-nums shrink-0">
                                            "+15 pts"
                                        </span>
                                    </li>
                                </ul>
                            </section>

                            // Botón para cerrar sesión desde el perfil (móvil y escritorio)
                            <div class="pt-1 pb-3">
                                <button
                                    type="button"
                                    class="w-full py-3 px-4 rounded-[16px] text-xs font-semibold text-[#FF3B30] bg-[#FF3B30]/10 hover:bg-[#FF3B30]/15 active:scale-[0.99] transition flex items-center justify-center space-x-2 cursor-pointer"
                                    on:click=move |_| {
                                        crate::storage::logout_and_clear_session(current_user);
                                        show_modal.set(false);
                                    }
                                >
                                    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                        <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/>
                                        <polyline points="16 17 21 12 16 7"/>
                                        <line x1="21" y1="12" x2="9" y2="12"/>
                                    </svg>
                                    <span>"Cerrar Sesión"</span>
                                </button>
                            </div>
                        }.into_view()
                    } else {
                        view! {
                            <div class="py-12 text-center text-[13px] text-[#86868B]">
                                "Inicia sesión para consultar tu progreso ciudadano."
                            </div>
                        }.into_view()
                    }
                }}
            </div>
        </dialog>
    }
}
