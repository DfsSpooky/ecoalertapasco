use leptos::*;
use crate::api::confirm_alert_url;
use crate::models::{AuthUser, ConfirmResponse, EcoAlert, EcoCategory, EcoSeverity};

fn render_category_svg(cat: EcoCategory) -> View {
    match cat {
        EcoCategory::Mineria => view! {
            <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.1" stroke-linecap="round" stroke-linejoin="round">
                <path d="m14 12-8.5 8.5a2.12 2.12 0 1 1-3-3L11 9"/>
                <path d="M15 13 9 7l4-4 6 6-4 4Z"/>
                <path d="m18 10 3 3"/>
            </svg>
        }.into_view(),
        EcoCategory::Basura => view! {
            <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.1" stroke-linecap="round" stroke-linejoin="round">
                <path d="M3 6h18"/>
                <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/>
                <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/>
            </svg>
        }.into_view(),
        EcoCategory::Agua => view! {
            <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.1" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 22a7 7 0 0 0 7-7c0-2-1-3.9-3-5.5s-3.5-4-4-6.5c-.5 2.5-2 4.9-4 6.5C6 11.1 5 13 5 15a7 7 0 0 0 7 7z"/>
            </svg>
        }.into_view(),
        EcoCategory::Aire => view! {
            <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.1" stroke-linecap="round" stroke-linejoin="round">
                <path d="M17.7 7.7a2.5 2.5 0 1 1 1.8 4.3H2"/>
                <path d="M9.6 4.6A2 2 0 1 1 11 8H2"/>
                <path d="M12.6 19.4A2 2 0 1 0 14 16H2"/>
            </svg>
        }.into_view(),
    }
}

pub fn render_severity_badge(sev: EcoSeverity) -> View {
    match sev {
        EcoSeverity::Critico => view! {
            <span class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-red-600 text-white text-[11px] font-bold shadow-xs tracking-wide">
                <svg class="w-3.5 h-3.5 text-white shrink-0" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M8.485 2.495c.673-1.167 2.357-1.167 3.03 0l6.28 10.875c.673 1.167-.17 2.625-1.516 2.625H3.72c-2.309 0-3.752-2.5-2.598-4.5L9.4 3.003ZM10 5a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-1.5 0v-3.5A.75.75 0 0 1 10 5Zm0 9a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z" clip-rule="evenodd" />
                </svg>
                <span>"Crítico"</span>
            </span>
        }.into_view(),
        EcoSeverity::Medio => view! {
            <span class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-amber-500 text-white text-[11px] font-bold shadow-xs tracking-wide">
                <svg class="w-3.5 h-3.5 text-white shrink-0" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0Zm-8-5a.75.75 0 0 1 .75.75v4.5a.75.75 0 0 1-1.5 0v-4.5A.75.75 0 0 1 10 5Zm0 10a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z" clip-rule="evenodd" />
                </svg>
                <span>"Medio"</span>
            </span>
        }.into_view(),
        EcoSeverity::Bajo => view! {
            <span class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-emerald-600 text-white text-[11px] font-bold shadow-xs tracking-wide">
                <svg class="w-3.5 h-3.5 text-white shrink-0" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M10 18a8 8 0 1 0 0-16 8 8 0 0 0 0 16Zm3.857-9.809a.75.75 0 0 0-1.214-.882l-3.483 4.79-1.88-1.88a.75.75 0 1 0-1.06 1.061l2.5 2.5a.75.75 0 0 0 1.137-.089l4-5.5Z" clip-rule="evenodd" />
                </svg>
                <span>"Bajo"</span>
            </span>
        }.into_view(),
    }
}

fn render_history_node_svg(action_str: &str) -> (&'static str, &'static str, View) {
    if action_str.contains("Reportado") || action_str.contains("creado") || action_str.contains("recibido") {
        (
            "bg-[#FF9500]/12",
            "text-[#C93400]",
            view! {
                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M4 15s1-1 4-1 5 2 8 2 4-1 4-1V3s-1 1-4 1-5-2-8-2-4 1-4 1z"/>
                    <line x1="4" y1="22" x2="4" y2="15"/>
                </svg>
            }.into_view(),
        )
    } else if action_str.contains("Cuadrilla") || action_str.contains("Inspección") || action_str.contains("Asignada") || action_str.contains("Validado") {
        (
            "bg-[#5856D6]/12",
            "text-[#4240B3]",
            view! {
                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="11" cy="11" r="8"/>
                    <path d="m21 21-4.3-4.3"/>
                </svg>
            }.into_view(),
        )
    } else if action_str.contains("Transferido") || action_str.contains("Jurisdicción") {
        (
            "bg-[#007AFF]/12",
            "text-[#0055D4]",
            view! {
                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M16 3h5v5"/>
                    <path d="M8 21H3v-5"/>
                    <path d="M21 3 14 10"/>
                    <path d="M3 21l7-7"/>
                </svg>
            }.into_view(),
        )
    } else if action_str.contains("Solucionado") || action_str.contains("Resuelto") || action_str.contains("Atendido") || action_str.contains("resuelta") || action_str.contains("cerrada") {
        (
            "bg-[#34C759]/14",
            "text-[#00704E]",
            view! {
                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="20 6 9 17 4 12"></polyline>
                </svg>
            }.into_view(),
        )
    } else if action_str.contains("Descartado") {
        (
            "bg-black/[0.06]",
            "text-[#6E6E73]",
            view! {
                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                    <line x1="18" y1="6" x2="6" y2="18"></line>
                    <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
            }.into_view(),
        )
    } else {
        (
            "bg-[#007AFF]/12",
            "text-[#007AFF]",
            view! {
                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="12" cy="12" r="10"/>
                    <polyline points="12 6 12 12 16 14"/>
                </svg>
            }.into_view(),
        )
    }
}

#[component]
pub fn AlertDetail(
    alerts: RwSignal<Vec<EcoAlert>>,
    selected_alert_id: RwSignal<Option<String>>,
    current_user: RwSignal<Option<AuthUser>>,
    show_auth_dialog: RwSignal<bool>,
    inspection_target: RwSignal<Option<EcoAlert>>,
) -> impl IntoView {
    let is_confirming = create_rw_signal(false);
    let lightbox_image = create_rw_signal(None::<String>);
    let show_history_modal = create_rw_signal(false);

    let history_dialog_ref = create_node_ref::<leptos::html::Dialog>();
    let history_pressed_outside = create_rw_signal(false);

    let selected_alert = create_memo(move |_| {
        let id_opt = selected_alert_id.get();
        id_opt.and_then(|id| alerts.get().into_iter().find(|a| a.id == id))
    });

    let active_alert_id = create_memo(move |_| {
        selected_alert.with(|opt| opt.as_ref().map(|a| a.id.clone()))
    });

    // Sync Bitácora <dialog> with show_history_modal signal
    create_effect(move |_| {
        let should_open = show_history_modal.get();
        if let Some(dialog_el) = history_dialog_ref.get() {
            if should_open {
                if !dialog_el.open() {
                    let _ = dialog_el.show_modal();
                }
            } else if dialog_el.open() {
                dialog_el.close();
            }
        }
    });

    // Close history modal automatically if selected alert is cleared
    create_effect(move |_| {
        if selected_alert_id.get().is_none() && show_history_modal.get_untracked() {
            show_history_modal.set(false);
        }
    });

    let is_pointer_outside = |ev: &web_sys::PointerEvent, el: &web_sys::HtmlDialogElement| -> bool {
        let r = el.get_bounding_client_rect();
        let x = ev.client_x() as f64;
        let y = ev.client_y() as f64;
        x < r.left() || x > r.right() || y < r.top() || y > r.bottom()
    };

    let toggle_confirm = move |alert_id: String| {
        let user_opt = current_user.get();
        if user_opt.is_none() {
            show_auth_dialog.set(true);
            return;
        }

        let user = user_opt.unwrap();
        is_confirming.set(true);

        spawn_local(async move {
            let url = confirm_alert_url(&alert_id);
            let req_res = gloo_net::http::Request::post(&url)
                .header("Authorization", &format!("Token {}", user.token))
                .header("Content-Type", "application/json")
                .send()
                .await;

            is_confirming.set(false);

            if let Ok(resp) = req_res {
                if resp.ok() {
                    if let Ok(data) = resp.json::<ConfirmResponse>().await {
                        alerts.update(|list| {
                            if let Some(target) = list.iter_mut().find(|a| a.id == alert_id) {
                                target.has_user_confirmed = data.confirmed;
                                target.confirmations_count = data.confirmations_count;
                            }
                        });
                    }
                }
            }
        });
    };

    view! {
        // Apple Maps Floating Inspector Card (Top-Right on Desktop to clear Zoom & Carousel; Bottom Sheet on Mobile)
        // Keyed on active_alert_id so <article> only animates on case selection/switch, never on background polls or +1 clicks
        {move || active_alert_id.get().and_then(|aid| {
            let a = selected_alert.get_untracked()?;
            let severity_enum = a.severity;

            let formatted_date = a.created_at.split('T').next().unwrap_or(&a.created_at).to_string();
            let reporter = a.reporter_name.clone().unwrap_or_else(|| "Ciudadano".to_string());
            let title_text = a.title.clone();
            let address_text = a.address.clone();
            let district_label = a.district.display_name();
            let category_enum = a.category;

            let photo1 = a.image_url.clone();
            let photo2 = a.image_url_2.clone();

            let is_solved_live = move || {
                selected_alert.with(|opt| opt.as_ref().map(|cur| cur.status == "solved").unwrap_or(false))
            };
            let has_confirmed_live = move || {
                selected_alert.with(|opt| opt.as_ref().map(|cur| cur.has_user_confirmed).unwrap_or(false))
            };
            let conf_count_live = move || {
                selected_alert.with(|opt| opt.as_ref().map(|cur| cur.confirmations_count).unwrap_or(0))
            };

            Some(view! {
                <article
                    aria-label="Detalle del caso ambiental"
                    class="apple-inspector-card fixed inset-x-0 bottom-0 md:inset-x-auto md:bottom-auto md:top-14 md:right-4 w-full md:w-[404px] max-h-[82vh] md:max-h-[calc(100vh-4.75rem)] z-40 bg-white/94 backdrop-blur-2xl backdrop-saturate-[1.8] rounded-t-[26px] md:rounded-[24px] shadow-[0_-16px_48px_rgba(0,0,0,0.16)] md:shadow-[0_24px_64px_-12px_rgba(0,0,0,0.22),0_4px_18px_rgba(0,0,0,0.06),0_0_0_0.5px_rgba(0,0,0,0.10),inset_0_1px_0_rgba(255,255,255,0.85)] flex flex-col overflow-hidden select-none"
                >
                    // Mobile Bottom-Sheet Grabber
                    <div class="pt-2.5 pb-0.5 flex justify-center md:hidden shrink-0">
                        <div class="w-9 h-1 bg-black/15 rounded-full"></div>
                    </div>

                    // 1. Header Region
                    <header class="px-5 pt-3.5 md:pt-4 pb-3 shrink-0">
                        <div class="flex items-start justify-between gap-3">
                            <div class="flex flex-wrap items-center gap-1.5 min-w-0">
                                {render_severity_badge(severity_enum)}

                                <span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full bg-black/[0.045] text-[#3C3C43] text-[11px] font-medium">
                                    {render_category_svg(category_enum)}
                                    <span>{category_enum.display_name()}</span>
                                </span>

                                {move || is_solved_live().then(|| view! {
                                    <span class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full bg-[#34C759]/14 text-[#00704E] ring-1 ring-inset ring-[#34C759]/28 text-[11px] font-semibold">
                                        <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round">
                                            <polyline points="20 6 9 17 4 12"></polyline>
                                        </svg>
                                        "Resuelto"
                                    </span>
                                })}
                            </div>

                            <button
                                type="button"
                                aria-label="Cerrar tarjeta del caso"
                                class="w-7 h-7 rounded-full bg-black/[0.06] hover:bg-black/[0.11] active:scale-95 text-[#3C3C43]/75 hover:text-[#1D1D1F] flex items-center justify-center transition-all shrink-0 cursor-pointer focus:outline-none focus-visible:ring-2 focus-visible:ring-[#007AFF]/50"
                                on:click=move |_| selected_alert_id.set(None)
                            >
                                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round">
                                    <line x1="18" y1="6" x2="6" y2="18"></line>
                                    <line x1="6" y1="6" x2="18" y2="18"></line>
                                </svg>
                            </button>
                        </div>

                        <h3 class="text-[16.5px] font-semibold tracking-[-0.022em] text-[#1D1D1F] leading-[1.3] mt-2">
                            {title_text.clone()}
                        </h3>
                    </header>

                    // 2. Scrollable Content Region
                    <div class="px-5 pb-3.5 overflow-y-auto overscroll-contain space-y-3.5 flex-1 min-h-0">
                        <p class="text-[13px] text-[#3C3C43]/90 leading-[1.5]">
                            {a.description}
                        </p>

                        // Citizen Photo Gallery (1 or 2 photos)
                        {match (photo1.clone(), photo2.clone()) {
                            (Some(u1), Some(u2)) => {
                                let u1_c = u1.clone();
                                let u2_c = u2.clone();
                                view! {
                                    <div class="grid grid-cols-2 gap-2">
                                        <button
                                            type="button"
                                            aria-label="Ampliar evidencia fotográfica 1"
                                            class="relative aspect-[4/3] w-full rounded-[16px] overflow-hidden bg-[#F5F5F7] ring-1 ring-black/[0.06] cursor-pointer group focus:outline-none focus-visible:ring-2 focus-visible:ring-[#007AFF]"
                                            on:click=move |_| lightbox_image.set(Some(u1_c.clone()))
                                        >
                                            <img src=u1 alt="Evidencia fotográfica 1" class="w-full h-full object-cover group-hover:scale-[1.03] transition-transform duration-300" />
                                            <span class="absolute bottom-2 right-2 px-2 py-0.5 rounded-full bg-black/55 backdrop-blur-md text-[10px] font-medium text-white flex items-center gap-1">
                                                <svg class="w-2.5 h-2.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round">
                                                    <polyline points="15 3 21 3 21 9"/><polyline points="9 21 3 21 3 15"/>
                                                    <line x1="21" y1="3" x2="14" y2="10"/><line x1="3" y1="21" x2="10" y2="14"/>
                                                </svg>
                                                "Foto 1"
                                            </span>
                                        </button>
                                        <button
                                            type="button"
                                            aria-label="Ampliar evidencia fotográfica 2"
                                            class="relative aspect-[4/3] w-full rounded-[16px] overflow-hidden bg-[#F5F5F7] ring-1 ring-black/[0.06] cursor-pointer group focus:outline-none focus-visible:ring-2 focus-visible:ring-[#007AFF]"
                                            on:click=move |_| lightbox_image.set(Some(u2_c.clone()))
                                        >
                                            <img src=u2 alt="Evidencia fotográfica 2" class="w-full h-full object-cover group-hover:scale-[1.03] transition-transform duration-300" />
                                            <span class="absolute bottom-2 right-2 px-2 py-0.5 rounded-full bg-black/55 backdrop-blur-md text-[10px] font-medium text-white flex items-center gap-1">
                                                <svg class="w-2.5 h-2.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round">
                                                    <polyline points="15 3 21 3 21 9"/><polyline points="9 21 3 21 3 15"/>
                                                    <line x1="21" y1="3" x2="14" y2="10"/><line x1="3" y1="21" x2="10" y2="14"/>
                                                </svg>
                                                "Foto 2"
                                            </span>
                                        </button>
                                    </div>
                                }.into_view()
                            }
                            (Some(u1), None) | (None, Some(u1)) => {
                                let u1_c = u1.clone();
                                view! {
                                    <button
                                        type="button"
                                        aria-label="Ampliar fotografía de evidencia"
                                        class="relative aspect-[16/9] w-full rounded-[16px] overflow-hidden bg-[#F5F5F7] ring-1 ring-black/[0.06] cursor-pointer group block focus:outline-none focus-visible:ring-2 focus-visible:ring-[#007AFF]"
                                        on:click=move |_| lightbox_image.set(Some(u1_c.clone()))
                                    >
                                        <img src=u1 alt="Evidencia fotográfica del incidente" class="w-full h-full object-cover group-hover:scale-[1.03] transition-transform duration-300" />
                                        <span class="absolute bottom-2 right-2 px-2.5 py-1 rounded-full bg-black/55 backdrop-blur-md text-[10.5px] font-medium text-white flex items-center gap-1">
                                            <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.3" stroke-linecap="round" stroke-linejoin="round">
                                                <polyline points="15 3 21 3 21 9"/><polyline points="9 21 3 21 3 15"/>
                                                <line x1="21" y1="3" x2="14" y2="10"/><line x1="3" y1="21" x2="10" y2="14"/>
                                            </svg>
                                            "Ampliar"
                                        </span>
                                    </button>
                                }.into_view()
                            }
                            (None, None) => view! { <span class="hidden"></span> }.into_view(),
                        }}

                        // Municipal Remediation Banner (When Solved)
                        {move || selected_alert.get().and_then(|cur| {
                            if cur.status != "solved" {
                                return None;
                            }
                            let resolved_date = cur
                                .resolved_at
                                .as_deref()
                                .map(|s| s.split('T').next().unwrap_or(s).to_string())
                                .unwrap_or_else(|| "Concluido".to_string());
                            let resolution_comment = cur.resolution_comment.clone();
                            let res_img_opt = cur.resolution_image_url.clone();

                            Some(view! {
                                <div class="rounded-[16px] bg-[#34C759]/[0.09] ring-1 ring-inset ring-[#34C759]/22 p-3.5 space-y-2">
                                    <div class="flex items-center justify-between gap-2">
                                        <span class="text-[11px] font-semibold text-[#00704E] flex items-center gap-1.5">
                                            <svg class="w-3.5 h-3.5 text-[#34C759]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                                                <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
                                                <path d="m9 12 2 2 4-4"/>
                                            </svg>
                                            "Remediación Municipal"
                                        </span>
                                        <span class="text-[11px] text-[#00704E]/80 tabular-nums font-medium">{resolved_date}</span>
                                    </div>
                                    {resolution_comment.map(|comm| view! {
                                        <p class="text-[12px] text-[#1D1D1F] leading-relaxed">{comm}</p>
                                    })}
                                    {res_img_opt.map(|u| {
                                        let u_clone = u.clone();
                                        view! {
                                            <button
                                                type="button"
                                                aria-label="Ampliar fotografía de remediación municipal"
                                                class="relative aspect-[16/9] w-full rounded-[12px] overflow-hidden ring-1 ring-[#34C759]/25 cursor-pointer group block"
                                                on:click=move |_| lightbox_image.set(Some(u_clone.clone()))
                                            >
                                                <img src=u class="w-full h-full object-cover group-hover:scale-[1.03] transition-transform duration-300" alt="Obra de remediación municipal" />
                                                <span class="absolute bottom-2 right-2 px-2.5 py-0.5 rounded-full bg-black/55 backdrop-blur-md text-[10px] font-medium text-white">
                                                    "Obra concluida"
                                                </span>
                                            </button>
                                        }
                                    })}
                                </div>
                            })
                        })}

                        // Apple Maps Inset Metadata Group (Location + Reporter + Date)
                        <div class="rounded-[16px] bg-[#F5F5F7]/95 ring-1 ring-inset ring-black/[0.045] divide-y divide-black/[0.055]">
                            <div class="px-3.5 py-2.5 flex items-start gap-2.5">
                                <span class="w-6 h-6 rounded-[7px] bg-[#007AFF]/12 text-[#007AFF] flex items-center justify-center shrink-0 mt-0.5">
                                    <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                                        <path d="M20 10c0 6-8 12-8 12s-8-6-8-12a8 8 0 0 1 16 0Z"/>
                                        <circle cx="12" cy="10" r="3"/>
                                    </svg>
                                </span>
                                <div class="min-w-0 flex-1">
                                    <p class="text-[12px] font-medium text-[#1D1D1F] leading-snug line-clamp-2">
                                        {address_text.clone()}
                                    </p>
                                    <p class="text-[11px] text-[#86868B] mt-0.5">
                                        {format!("Distrito de {}", district_label)}
                                    </p>
                                </div>
                            </div>

                            <div class="px-3.5 py-2 flex items-center justify-between gap-2 text-[11.5px]">
                                <span class="text-[#86868B] flex items-center gap-1.5 min-w-0 truncate">
                                    <svg class="w-3.5 h-3.5 text-[#86868B] shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                        <path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/>
                                        <circle cx="12" cy="7" r="4"/>
                                    </svg>
                                    <span class="truncate">"Por " <strong class="font-medium text-[#3C3C43]">{reporter}</strong></span>
                                </span>
                                <span class="text-[#86868B] tabular-nums shrink-0">{formatted_date}</span>
                            </div>
                        </div>
                    </div>

                    // 3. Pinned Apple Maps Action Footer (100% Visible, Zero Horizontal Overflow)
                    <footer class="px-5 pt-3 pb-[max(1rem,env(safe-area-inset-bottom))] md:pb-4 bg-white/88 backdrop-blur-xl border-t border-black/[0.06] space-y-2 shrink-0">
                        // Primary Civic Endorsement Button + Live Count Capsule
                        <button
                            type="button"
                            class=move || {
                                if has_confirmed_live() {
                                    "w-full h-10 px-3.5 rounded-[13px] bg-[#34C759]/14 hover:bg-[#34C759]/22 active:scale-[0.985] text-[#00704E] ring-1 ring-inset ring-[#34C759]/32 flex items-center justify-between gap-2 transition-all duration-200 cursor-pointer"
                                } else {
                                    "w-full h-10 px-3.5 rounded-[13px] bg-[#1D1D1F] hover:bg-[#2C2C2E] active:scale-[0.985] text-white shadow-[0_3px_10px_rgba(0,0,0,0.16)] flex items-center justify-between gap-2 transition-all duration-200 cursor-pointer"
                                }
                            }
                            disabled=move || is_confirming.get()
                            on:click={
                                let target_id = aid.clone();
                                move |_| toggle_confirm(target_id.clone())
                            }
                        >
                            <span class="flex items-center gap-2 text-[13px] font-semibold tracking-[-0.01em]">
                                {move || {
                                    let confirmed = has_confirmed_live();
                                    if is_confirming.get() {
                                        view! {
                                            <span class={if confirmed {
                                                "inline-block w-3.5 h-3.5 border-2 border-[#00704E]/25 border-t-[#00704E] rounded-full animate-spin"
                                            } else {
                                                "inline-block w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin"
                                            }}></span>
                                            <span>"Sincronizando…"</span>
                                        }.into_view()
                                    } else if confirmed {
                                        view! {
                                            <svg class="w-4 h-4 text-[#00825A] shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                                                <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
                                                <polyline points="22 4 12 14.01 9 11.01"/>
                                            </svg>
                                            <span>"Caso respaldado"</span>
                                        }.into_view()
                                    } else {
                                        view! {
                                            <svg class="w-4 h-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.1" stroke-linecap="round" stroke-linejoin="round">
                                                <path d="M7 10v12"/>
                                                <path d="M15 5.88 14 10h5.83a2 2 0 0 1 1.92 2.56l-2.33 8A2 2 0 0 1 17.5 22H4a2 2 0 0 1-2-2v-8a2 2 0 0 1 2-2h2.76a2 2 0 0 0 1.79-1.11L12 2a3.13 3.13 0 0 1 3 3.88Z"/>
                                            </svg>
                                            <span>"Respaldar este caso"</span>
                                        }.into_view()
                                    }
                                }}
                            </span>

                            <span class=move || {
                                if has_confirmed_live() {
                                    "px-2.5 py-0.5 rounded-full bg-[#34C759]/22 text-[#006442] text-[11px] font-semibold tabular-nums shrink-0"
                                } else {
                                    "px-2.5 py-0.5 rounded-full bg-white/15 text-white/95 text-[11px] font-semibold tabular-nums shrink-0"
                                }
                            }>
                                {move || {
                                    let c = conf_count_live();
                                    if c == 1 {
                                        "1 respaldo".to_string()
                                    } else {
                                        format!("{} respaldos", c)
                                    }
                                }}
                            </span>
                        </button>

                        // 3-Column Equal-Width Apple Maps Quick Actions (Acta, Bitácora, WhatsApp)
                        <div class="apple-inspector-actions grid grid-cols-3 gap-2">
                            <button
                                type="button"
                                class="h-9 px-2.5 rounded-[11px] bg-[#F5F5F7] hover:bg-[#E8E8ED] active:scale-[0.97] text-[#1D1D1F] ring-1 ring-inset ring-black/[0.05] flex items-center justify-center gap-1.5 text-[12px] font-semibold tracking-[-0.01em] transition-all cursor-pointer"
                                title="Generar o imprimir Acta Oficial de Fiscalización"
                                on:click=move |_| {
                                    if let Some(cur) = selected_alert.get_untracked() {
                                        inspection_target.set(Some(cur));
                                    }
                                }
                            >
                                <svg class="w-3.5 h-3.5 text-[#007AFF] shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                                    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                                    <polyline points="14 2 14 8 20 8"/>
                                    <path d="m9 15 2 2 4-4"/>
                                </svg>
                                <span>"Acta"</span>
                            </button>

                            <button
                                type="button"
                                class="h-9 px-2.5 rounded-[11px] bg-[#F5F5F7] hover:bg-[#E8E8ED] active:scale-[0.97] text-[#1D1D1F] ring-1 ring-inset ring-black/[0.05] flex items-center justify-center gap-1.5 text-[12px] font-semibold tracking-[-0.01em] transition-all cursor-pointer"
                                title="Ver bitácora de intervenciones y auditoría de la denuncia"
                                on:click=move |_| show_history_modal.set(true)
                            >
                                <svg class="w-3.5 h-3.5 text-[#5856D6] shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                                    <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
                                    <path d="M3 3v5h5"/>
                                    <path d="M12 7v5l3 2"/>
                                </svg>
                                <span>"Bitácora"</span>
                            </button>

                            <button
                                type="button"
                                class="h-9 px-2.5 rounded-[11px] bg-[#34C759]/[0.10] hover:bg-[#34C759]/[0.18] active:scale-[0.97] text-[#00704E] ring-1 ring-inset ring-[#34C759]/24 flex items-center justify-center gap-1.5 text-[12px] font-semibold tracking-[-0.01em] transition-all cursor-pointer"
                                title="Compartir alerta en grupos de WhatsApp"
                                on:click={
                                    let t = title_text.clone();
                                    let ad = address_text.clone();
                                    let id = aid.clone();
                                    move |_| {
                                        let text = format!(
                                            "📢 *EcoAlerta Pasco*: Alerta ambiental reportada en Cerro de Pasco.\n📍 *Lugar:* {}\n⚠️ *Asunto:* {}\n🔍 *Código:* EXP-2026-{}\n\nIngresa a EcoAlerta para respaldar con tu +1 y acelerar la atención municipal.",
                                            ad, t, id
                                        );
                                        let encoded = js_sys::encode_uri_component(&text);
                                        let wa_url = format!("https://wa.me/?text={}", encoded.as_string().unwrap_or_default());
                                        let _ = js_sys::eval(&format!("window.open('{}', '_blank');", wa_url));
                                    }
                                }
                            >
                                <i class="fa-brands fa-whatsapp text-[#25D366] text-[14px] shrink-0"></i>
                                <span>"WhatsApp"</span>
                            </button>
                        </div>
                    </footer>
                </article>
            })
        })}

        // Apple QuickLook-Style Image Lightbox
        {move || lightbox_image.get().map(|img_url| view! {
            <div
                class="fixed inset-0 z-50 bg-black/80 backdrop-blur-xl flex items-center justify-center p-4 animate-in fade-in duration-150 cursor-pointer"
                on:click=move |_| lightbox_image.set(None)
            >
                <div
                    class="relative max-w-3xl max-h-[88vh] bg-[#1C1C1E] rounded-[24px] overflow-hidden ring-1 ring-white/15 shadow-[0_32px_90px_rgba(0,0,0,0.65)] p-1.5"
                    on:click=move |e| e.stop_propagation()
                >
                    <img src=img_url alt="Evidencia ampliada" class="max-w-full max-h-[82vh] object-contain rounded-[18px]" />
                    <button
                        type="button"
                        aria-label="Cerrar imagen ampliada"
                        class="absolute top-3.5 right-3.5 w-8 h-8 rounded-full bg-black/55 hover:bg-black/80 backdrop-blur-md text-white/90 hover:text-white flex items-center justify-center ring-1 ring-white/15 transition-all cursor-pointer"
                        on:click=move |_| lightbox_image.set(None)
                    >
                        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round">
                            <line x1="18" y1="6" x2="6" y2="18"></line>
                            <line x1="6" y1="6" x2="18" y2="18"></line>
                        </svg>
                    </button>
                </div>
            </div>
        })}

        // Case History Timeline Modal (Native Top-Layer <dialog> in Apple Sequoia Style)
        <dialog
            node_ref=history_dialog_ref
            class="apple-modal-dialog apple-modal-dialog--sm select-none"
            aria-labelledby="alert-history-modal-title"
            attr:closedby="any"
            on:close=move |_| {
                history_pressed_outside.set(false);
                show_history_modal.set(false);
            }
            on:pointerdown=move |ev| {
                if let Some(el) = history_dialog_ref.get() {
                    history_pressed_outside.set(is_pointer_outside(&ev, &el));
                }
            }
            on:pointerup=move |ev| {
                if let Some(el) = history_dialog_ref.get() {
                    if history_pressed_outside.get_untracked() && is_pointer_outside(&ev, &el) {
                        el.close();
                    }
                    history_pressed_outside.set(false);
                }
            }
        >
            <div class="pt-2.5 pb-0.5 flex justify-center sm:hidden bg-[#F5F5F7]/90 shrink-0">
                <div class="w-9 h-1 rounded-full bg-black/15"></div>
            </div>

            <header class="px-5 pt-3.5 sm:pt-4 pb-3.5 bg-[#F5F5F7]/88 backdrop-blur-xl border-b border-black/[0.06] flex items-center justify-between gap-3 shrink-0">
                <div class="min-w-0">
                    <h2
                        id="alert-history-modal-title"
                        class="text-[16px] font-semibold tracking-[-0.02em] text-[#1D1D1F] leading-tight"
                    >
                        "Bitácora del Caso"
                    </h2>
                    <p class="text-[11.5px] text-[#86868B] mt-0.5 truncate">
                        "Trazabilidad e intervenciones registradas"
                    </p>
                </div>

                <button
                    type="button"
                    autofocus
                    aria-label="Cerrar bitácora del caso"
                    class="w-7 h-7 rounded-full bg-black/[0.06] hover:bg-black/[0.11] active:scale-95 text-[#3C3C43]/75 hover:text-[#1D1D1F] flex items-center justify-center transition-all shrink-0 cursor-pointer"
                    on:click=move |_| show_history_modal.set(false)
                >
                    <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round">
                        <line x1="18" y1="6" x2="6" y2="18"></line>
                        <line x1="6" y1="6" x2="18" y2="18"></line>
                    </svg>
                </button>
            </header>

            {move || selected_alert.get().map(|curr_alert| {
                let history_items = curr_alert.parsed_history();
                view! {
                    <div class="p-5 overflow-y-auto overscroll-contain space-y-4 flex-1 min-h-0">
                        <div class="p-3.5 bg-white rounded-[16px] shadow-[0_1px_6px_rgba(0,0,0,0.03),0_0_0_0.5px_rgba(0,0,0,0.06)]">
                            <p class="text-[13px] font-semibold text-[#1D1D1F] leading-snug">{curr_alert.title.clone()}</p>
                            <p class="text-[11.5px] text-[#86868B] mt-0.5 truncate">{curr_alert.address.clone()}</p>
                        </div>

                        <div class="bg-white rounded-[18px] p-4 shadow-[0_1px_6px_rgba(0,0,0,0.03),0_0_0_0.5px_rgba(0,0,0,0.06)]">
                            {if history_items.is_empty() {
                                view! {
                                    <div class="py-6 text-center space-y-2">
                                        <div class="w-10 h-10 rounded-full bg-[#F5F5F7] text-[#86868B] mx-auto flex items-center justify-center">
                                            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round">
                                                <circle cx="12" cy="12" r="10"/>
                                                <polyline points="12 6 12 12 16 14"/>
                                            </svg>
                                        </div>
                                        <p class="text-[12.5px] font-semibold text-[#1D1D1F]">"Recepción inicial registrada"</p>
                                        <p class="text-[11.5px] text-[#86868B] leading-relaxed max-w-xs mx-auto">
                                            "Aún no se registran movimientos de cuadrilla. Las actuaciones municipales aparecerán aquí."
                                        </p>
                                    </div>
                                }.into_view()
                            } else {
                                let total_items = history_items.len();
                                view! {
                                    <div class="space-y-0">
                                        {history_items.into_iter().enumerate().map(|(idx, (date_str, action_str))| {
                                            let (bg_cls, text_cls, node_svg) = render_history_node_svg(&action_str);
                                            let is_last = idx == total_items - 1;

                                            view! {
                                                <div class="flex items-stretch gap-3">
                                                    <div class="flex flex-col items-center">
                                                        <div class=format!("w-7 h-7 rounded-full flex items-center justify-center shrink-0 {} {}", bg_cls, text_cls)>
                                                            {node_svg}
                                                        </div>
                                                        {(!is_last).then(|| view! {
                                                            <div class="w-px flex-1 bg-black/[0.08] my-1 min-h-[18px]"></div>
                                                        })}
                                                    </div>
                                                    <div class=if is_last { "flex-1 pb-0.5" } else { "flex-1 pb-3.5" }>
                                                        <span class="block text-[10.5px] text-[#86868B] tabular-nums font-medium">
                                                            {date_str}
                                                        </span>
                                                        <p class="text-[12.5px] font-medium text-[#1D1D1F] mt-0.5 leading-snug">
                                                            {action_str}
                                                        </p>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_view()
                            }}
                        </div>
                    </div>
                }
            })}
        </dialog>
    }
}
