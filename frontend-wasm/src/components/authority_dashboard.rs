use leptos::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use crate::api::{
    alert_detail_url, system_settings_url, test_gemini_url, upload_url, waste_point_detail_url,
    waste_points_url,
};
use crate::components::fleet_routes_tab::FleetRoutesTab;
use crate::models::{
    TruckRun,
    AuthUser, CreateWastePointPayload, EcoAlert, EcoCategory, EcoDistrict, EcoSeverity, FillLevel,
    GeminiModelInfo, SystemSettingsResponse, TestGeminiResponse, WastePoint, WasteType,
};

#[component]
pub fn AuthorityDashboard(
    alerts: RwSignal<Vec<EcoAlert>>,
    waste_points: RwSignal<Vec<WastePoint>>,
    show_dashboard: RwSignal<bool>,
    current_user: RwSignal<Option<AuthUser>>,
    show_auth_dialog: RwSignal<bool>,
    inspection_target: RwSignal<Option<EcoAlert>>,
    selected_alert_id: RwSignal<Option<String>>,
    selected_waste_point_id: RwSignal<Option<String>>,
    show_waste_layer: RwSignal<bool>,
    truck_runs: RwSignal<Vec<TruckRun>>,
) -> impl IntoView {
    let active_tab = create_rw_signal(0); // 0 = Denuncias, 1 = Puntos de Acopio, 2 = Rutas y Flota

    // Alert filters
    let search_query = create_rw_signal("".to_string());
    let filter_status = create_rw_signal("todos".to_string()); // "todos", "pending", "solved", "dismissed"
    let filter_district = create_rw_signal(None::<EcoDistrict>);
    let filter_severity = create_rw_signal(None::<EcoSeverity>);
    let filter_urgent_sla = create_rw_signal(false);
    let show_analytics = create_rw_signal(true);

    // Waste filters & state
    let waste_search_query = create_rw_signal("".to_string());
    let filter_waste_fill = create_rw_signal("todos".to_string()); // "todos", "full", "medium", "low"
    let filter_waste_type = create_rw_signal("todos".to_string()); // "todos", "recyclable", "organic", "general"
    let is_locating_waste_gps = create_rw_signal(false);
    let is_reverse_geocoding_waste = create_rw_signal(false);
    let show_manual_coords_details = create_rw_signal(false);

    // Modals state
    let resolve_target = create_rw_signal(None::<EcoAlert>);
    let resolve_comment = create_rw_signal("".to_string());
    let resolve_photo_url = create_rw_signal(None::<String>);
    let is_uploading_resolve_photo = create_rw_signal(false);
    let is_submitting_action = create_rw_signal(false);
    let action_error = create_rw_signal(None::<String>);

    let dismiss_target = create_rw_signal(None::<EcoAlert>);
    let dismiss_reason = create_rw_signal("".to_string());

    let transfer_target = create_rw_signal(None::<EcoAlert>);
    let transfer_district = create_rw_signal(EcoDistrict::Chaupimarca);

    // Assign squad modal state
    let assign_squad_target = create_rw_signal(None::<EcoAlert>);
    let selected_squad = create_rw_signal("Cuadrilla de Limpieza y Residuos Sólidos".to_string());
    let inspector_name = create_rw_signal("Ing. Fiscalizador Ambiental".to_string());
    let squad_notes = create_rw_signal("Inspección técnica inmediata y ejecución de medidas correctivas.".to_string());

    // History timeline modal state
    let history_target = create_rw_signal(None::<EcoAlert>);

    // Gemini AI modal state
    let show_gemini_modal = create_rw_signal(false);
    let gemini_settings = create_rw_signal(None::<SystemSettingsResponse>);
    let gemini_key_input = create_rw_signal("".to_string());
    let gemini_model_input = create_rw_signal("gemini-3.5-flash".to_string());
    let gemini_enabled_input = create_rw_signal(true);
    let turnstile_sitekey_input = create_rw_signal("".to_string());
    let turnstile_secret_input = create_rw_signal("".to_string());
    let turnstile_enabled_input = create_rw_signal(true);
    let gemini_test_result = create_rw_signal(None::<TestGeminiResponse>);
    let is_loading_settings = create_rw_signal(false);
    let is_saving_settings = create_rw_signal(false);
    let is_testing_gemini = create_rw_signal(false);
    let gemini_status_msg = create_rw_signal(None::<(bool, String)>);

    // Waste point modal state (Create & Edit)
    let show_create_waste_modal = create_rw_signal(false);
    let editing_waste_target = create_rw_signal(None::<WastePoint>);
    let new_waste_name = create_rw_signal("".to_string());
    let new_waste_desc = create_rw_signal("".to_string());
    let new_waste_addr = create_rw_signal("".to_string());
    let new_waste_lat = create_rw_signal(-10.6675);
    let new_waste_lng = create_rw_signal(-76.2567);
    let new_waste_type = create_rw_signal(WasteType::Recyclable);
    let new_waste_fill = create_rw_signal(FillLevel::Low);
    let new_waste_schedule = create_rw_signal("".to_string());
    let new_waste_recom = create_rw_signal("".to_string());
    let new_waste_img = create_rw_signal("".to_string());

    // SLA helper: returns (is_urgent, is_overdue, label, color_class)
    let get_alert_sla = |a: &EcoAlert| -> (bool, bool, &'static str, &'static str) {
        if a.status == "solved" {
            return (false, false, "Atendido", "bg-emerald-50 text-emerald-700 border-emerald-200");
        }
        if a.status == "dismissed" {
            return (false, false, "Descartado", "bg-slate-100 text-slate-500 border-slate-200");
        }
        if a.status == "verified" {
            return (false, false, "En Inspección", "bg-purple-50 text-purple-700 border-purple-200 font-semibold");
        }

        // Parse approximate age in hours from created_at
        let lower = a.created_at.to_lowercase();
        let hours = if lower.contains("día") || lower.contains("dia") {
            let num: f64 = lower
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .unwrap_or(1.0);
            num * 24.0
        } else if lower.contains("hora") {
            let num: f64 = lower
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .unwrap_or(1.0);
            num
        } else {
            2.0
        };

        if a.severity == EcoSeverity::Critico && hours >= 24.0 {
            (false, true, "🔴 Vencido (>24h)", "bg-red-50 text-red-700 border-red-200 font-bold animate-pulse")
        } else if hours >= 48.0 {
            (false, true, "🔴 Vencido (>48h)", "bg-red-50 text-red-700 border-red-200 font-bold")
        } else if hours >= 24.0 {
            (true, false, "🟡 Urgente (>24h)", "bg-amber-50 text-amber-700 border-amber-200 font-semibold")
        } else {
            (false, false, "🟢 En Plazo", "bg-emerald-50 text-emerald-700 border-emerald-200 font-medium")
        }
    };

    // Filtered alerts for the dashboard table
    let displayed_alerts = create_memo(move |_| {
        let all = alerts.get();
        let query = search_query.get().trim().to_lowercase();
        let stat = filter_status.get();
        let dist_opt = filter_district.get();
        let sev_opt = filter_severity.get();
        let only_urgent = filter_urgent_sla.get();

        all.into_iter()
            .filter(|a| {
                if stat != "todos" && a.status != stat {
                    return false;
                }
                if let Some(dist) = dist_opt {
                    if a.district != dist {
                        return false;
                    }
                }
                if let Some(sev) = sev_opt {
                    if a.severity != sev {
                        return false;
                    }
                }
                if only_urgent {
                    let (urg, over, _, _) = get_alert_sla(a);
                    if !urg && !over {
                        return false;
                    }
                }
                if !query.is_empty() {
                    let m_title = a.title.to_lowercase().contains(&query);
                    let m_desc = a.description.to_lowercase().contains(&query);
                    let m_addr = a.address.to_lowercase().contains(&query);
                    if !m_title && !m_desc && !m_addr {
                        return false;
                    }
                }
                true
            })
            .collect::<Vec<EcoAlert>>()
    });

    // KPI counts
    let total_count = move || alerts.get().len();
    let pending_count = move || alerts.get().iter().filter(|a| a.status == "pending").count();
    let verified_count = move || alerts.get().iter().filter(|a| a.status == "verified").count();
    let solved_count = move || alerts.get().iter().filter(|a| a.status == "solved").count();
    let urgent_count = move || {
        alerts.get().iter().filter(|a| {
            let (urg, over, _, _) = get_alert_sla(a);
            urg || over
        }).count()
    };

    // Filtered waste points for the dashboard
    let displayed_waste_points = create_memo(move |_| {
        let all = waste_points.get();
        let query = waste_search_query.get().trim().to_lowercase();
        let fill = filter_waste_fill.get();
        let wtype = filter_waste_type.get();

        all.into_iter()
            .filter(|wp| {
                if fill != "todos" && wp.fill_level.as_str() != fill {
                    return false;
                }
                if wtype != "todos" && wp.waste_type.as_str() != wtype {
                    return false;
                }
                if !query.is_empty() {
                    let m_name = wp.name.to_lowercase().contains(&query);
                    let m_desc = wp.description.to_lowercase().contains(&query);
                    let m_addr = wp.address.to_lowercase().contains(&query);
                    if !m_name && !m_desc && !m_addr {
                        return false;
                    }
                }
                true
            })
            .collect::<Vec<WastePoint>>()
    });

    let total_waste_count = move || waste_points.get().len();
    let full_waste_count = move || waste_points.get().iter().filter(|p| p.fill_level == FillLevel::Full).count();
    let medium_waste_count = move || waste_points.get().iter().filter(|p| p.fill_level == FillLevel::Medium).count();
    let low_waste_count = move || waste_points.get().iter().filter(|p| p.fill_level == FillLevel::Low).count();

    // Action 1: Submit Resolve
    let submit_resolve = move |_| {
        let user = match current_user.get() {
            Some(u) if u.is_authority() => u,
            Some(_) => {
                action_error.set(Some("Acceso denegado: Tu cuenta no tiene rol de autoridad fiscalizadora.".to_string()));
                return;
            }
            None => {
                show_auth_dialog.set(true);
                return;
            }
        };
        let target = match resolve_target.get() {
            Some(t) => t,
            None => return,
        };

        let comment = resolve_comment.get();
        if comment.trim().is_empty() {
            action_error.set(Some("Ingresa el informe o descripción del trabajo realizado.".to_string()));
            return;
        }

        is_submitting_action.set(true);
        action_error.set(None);

        let target_id = target.id.clone();
        let photo_url = resolve_photo_url.get();
        let token = user.token.clone();

        spawn_local(async move {
            let payload = serde_json::json!({
                "status": "solved",
                "resolution_comment": comment,
                "resolution_image_url": photo_url,
            });

            let resp = gloo_net::http::Request::patch(&alert_detail_url(&target_id))
                .header("Authorization", &format!("Token {}", token))
                .json(&payload);

            match resp {
                Ok(req) => match req.send().await {
                    Ok(res) => {
                        if res.ok() {
                            alerts.update(|list| {
                                if let Some(item) = list.iter_mut().find(|a| a.id == target_id) {
                                    item.status = "solved".to_string();
                                    item.resolution_comment = Some(comment);
                                    item.resolution_image_url = photo_url;
                                    item.resolved_at = Some(js_sys::Date::new_0().to_locale_string("es-PE", &js_sys::Object::new()).into());
                                }
                            });
                            resolve_target.set(None);
                        } else {
                            action_error.set(Some(format!("Error en el servidor al resolver ({})", res.status())));
                        }
                    }
                    Err(e) => {
                        action_error.set(Some(format!("Error de conexión: {}", e)));
                    }
                },
                Err(e) => {
                    action_error.set(Some(format!("Error serializando: {}", e)));
                }
            }
            is_submitting_action.set(false);
        });
    };

    // Action 2: Submit Dismiss
    let submit_dismiss = move |_| {
        let user = match current_user.get() {
            Some(u) if u.is_authority() => u,
            Some(_) => {
                action_error.set(Some("Acceso denegado: Tu cuenta no tiene rol de autoridad fiscalizadora.".to_string()));
                return;
            }
            None => {
                show_auth_dialog.set(true);
                return;
            }
        };
        let target = match dismiss_target.get() {
            Some(t) => t,
            None => return,
        };

        let reason = dismiss_reason.get();
        if reason.trim().is_empty() {
            action_error.set(Some("Ingresa el motivo técnico por el cual se desestima la alerta.".to_string()));
            return;
        }

        is_submitting_action.set(true);
        action_error.set(None);

        let target_id = target.id.clone();
        let token = user.token.clone();

        spawn_local(async move {
            let payload = serde_json::json!({
                "status": "dismissed",
                "resolution_comment": reason,
            });

            let resp = gloo_net::http::Request::patch(&alert_detail_url(&target_id))
                .header("Authorization", &format!("Token {}", token))
                .json(&payload);

            match resp {
                Ok(req) => match req.send().await {
                    Ok(res) => {
                        if res.ok() {
                            alerts.update(|list| {
                                if let Some(item) = list.iter_mut().find(|a| a.id == target_id) {
                                    item.status = "dismissed".to_string();
                                    item.resolution_comment = Some(reason);
                                }
                            });
                            dismiss_target.set(None);
                        } else {
                            action_error.set(Some(format!("Error descartando alerta ({})", res.status())));
                        }
                    }
                    Err(e) => action_error.set(Some(format!("Error de red: {}", e))),
                },
                Err(e) => action_error.set(Some(format!("Error: {}", e))),
            }
            is_submitting_action.set(false);
        });
    };

    // Action 3: Submit Transfer
    let submit_transfer = move |_| {
        let user = match current_user.get() {
            Some(u) if u.is_authority() => u,
            Some(_) => {
                action_error.set(Some("Acceso denegado: Tu cuenta no tiene rol de autoridad fiscalizadora.".to_string()));
                return;
            }
            None => {
                show_auth_dialog.set(true);
                return;
            }
        };
        let target = match transfer_target.get() {
            Some(t) => t,
            None => return,
        };

        let new_dist = transfer_district.get();
        is_submitting_action.set(true);
        action_error.set(None);

        let target_id = target.id.clone();
        let token = user.token.clone();

        spawn_local(async move {
            let payload = serde_json::json!({
                "district": new_dist,
            });

            let resp = gloo_net::http::Request::patch(&alert_detail_url(&target_id))
                .header("Authorization", &format!("Token {}", token))
                .json(&payload);

            match resp {
                Ok(req) => match req.send().await {
                    Ok(res) => {
                        if res.ok() {
                            alerts.update(|list| {
                                if let Some(item) = list.iter_mut().find(|a| a.id == target_id) {
                                    item.district = new_dist;
                                }
                            });
                            transfer_target.set(None);
                        } else {
                            action_error.set(Some(format!("Error transfiriendo jurisdicción ({})", res.status())));
                        }
                    }
                    Err(e) => action_error.set(Some(format!("Error de red: {}", e))),
                },
                Err(e) => action_error.set(Some(format!("Error: {}", e))),
            }
            is_submitting_action.set(false);
        });
    };

    // Action 3.5: Submit Assign Squad / Inspector
    let submit_assign_squad = move |_| {
        let user = match current_user.get() {
            Some(u) if u.is_authority() => u,
            Some(_) => {
                action_error.set(Some("Acceso denegado: Tu cuenta no tiene rol de autoridad fiscalizadora.".to_string()));
                return;
            }
            None => {
                show_auth_dialog.set(true);
                return;
            }
        };
        let target = match assign_squad_target.get() {
            Some(t) => t,
            None => return,
        };

        let squad = selected_squad.get();
        let inspector = inspector_name.get().trim().to_string();
        let notes = squad_notes.get().trim().to_string();
        let comment = format!("Cuadrilla Asignada: {}. Responsable: {} | Instrucciones: {}", squad, inspector, notes);

        is_submitting_action.set(true);
        action_error.set(None);

        let target_id = target.id.clone();
        let token = user.token.clone();

        spawn_local(async move {
            let payload = serde_json::json!({
                "status": "verified",
                "resolution_comment": comment,
            });

            let resp = gloo_net::http::Request::patch(&alert_detail_url(&target_id))
                .header("Authorization", &format!("Token {}", token))
                .json(&payload);

            match resp {
                Ok(req) => match req.send().await {
                    Ok(res) => {
                        if res.ok() {
                            alerts.update(|list| {
                                if let Some(item) = list.iter_mut().find(|a| a.id == target_id) {
                                    item.status = "verified".to_string();
                                    item.resolution_comment = Some(comment.clone());
                                    let now_str = js_sys::Date::new_0().to_iso_string().as_string().unwrap_or_else(|| "Hoy".to_string());
                                    let new_entry = format!("{} | Cuadrilla Asignada: {} ({})", now_str, squad, inspector);
                                    let updated_history = match &item.history_log {
                                        Some(h) if !h.trim().is_empty() => format!("{}\n{}", h, new_entry),
                                        _ => new_entry,
                                    };
                                    item.history_log = Some(updated_history);
                                }
                            });
                            assign_squad_target.set(None);
                        } else {
                            action_error.set(Some(format!("Error asignando cuadrilla en el servidor ({})", res.status())));
                        }
                    }
                    Err(e) => action_error.set(Some(format!("Error de conexión: {}", e))),
                },
                Err(e) => action_error.set(Some(format!("Error: {}", e))),
            }
            is_submitting_action.set(false);
        });
    };

    // Action 4: Quick Patch Waste Point Fill Level
    let update_waste_fill_level = move |pid: String, new_level: FillLevel| {
        let user = match current_user.get() {
            Some(u) if u.is_authority() => u,
            _ => return,
        };

        // Optimistically update locally
        waste_points.update(|list| {
            if let Some(p) = list.iter_mut().find(|item| item.id_string() == pid) {
                p.fill_level = new_level;
            }
        });

        // Patch to backend if authenticated
        let p_url = waste_point_detail_url(&pid);
        let token = user.token.clone();
        spawn_local(async move {
            let payload = serde_json::json!({
                "fill_level": new_level,
            });
            if let Ok(req) = gloo_net::http::Request::patch(&p_url)
                .header("Authorization", &format!("Token {}", token))
                .json(&payload)
            {
                let _ = req.send().await;
            }
        });
    };

    // Action 5: Delete Waste Point
    let delete_waste_point = move |pid: String| {
        let user = match current_user.get() {
            Some(u) if u.is_authority() => u,
            _ => return,
        };

        waste_points.update(|list| {
            list.retain(|p| p.id_string() != pid);
        });

        let p_url = waste_point_detail_url(&pid);
        let token = user.token.clone();
        spawn_local(async move {
            let _ = gloo_net::http::Request::delete(&p_url)
                .header("Authorization", &format!("Token {}", token))
                .send()
                .await;
        });
    };

    // Coordinate synchronization with Leaflet picker
    create_effect(move |_| {
        use wasm_bindgen::closure::Closure;
        let on_coords = Closure::wrap(Box::new(move |lat: f64, lng: f64| {
            new_waste_lat.set(lat);
            new_waste_lng.set(lng);
        }) as Box<dyn FnMut(f64, f64)>);

        let _ = js_sys::Reflect::set(
            &web_sys::window().unwrap(),
            &wasm_bindgen::JsValue::from_str("onWastePickerCoordsChanged"),
            on_coords.as_ref().unchecked_ref(),
        );
        on_coords.forget();
    });

    create_effect(move |_| {
        if show_create_waste_modal.get() {
            let lat = new_waste_lat.get_untracked();
            let lng = new_waste_lng.get_untracked();
            let _ = js_sys::eval(&format!(
                "if (window.ecoWastePicker) {{ window.ecoWastePicker.init('waste-picker-map', {}, {}); }}",
                lat, lng
            ));
        }
    });

    let locate_waste_gps = move |_| {
        is_locating_waste_gps.set(true);
        action_error.set(None);
        spawn_local(async move {
            match crate::geo::get_browser_gps_position().await {
                Ok((lat, lng)) => {
                    new_waste_lat.set(lat);
                    new_waste_lng.set(lng);
                    let _ = js_sys::eval(&format!(
                        "if (window.ecoWastePicker) window.ecoWastePicker.setPosition({}, {});",
                        lat, lng
                    ));
                    if new_waste_addr.get_untracked().trim().is_empty() {
                        let (addr_str, _) = crate::geo::reverse_geocode(lat, lng).await;
                        new_waste_addr.set(addr_str);
                    }
                }
                Err(err) => {
                    action_error.set(Some(format!("No se pudo obtener la posición GPS: {}", err)));
                }
            }
            is_locating_waste_gps.set(false);
        });
    };

    let set_waste_preset = move |name: &str, lat: f64, lng: f64| {
        new_waste_lat.set(lat);
        new_waste_lng.set(lng);
        if new_waste_addr.get_untracked().trim().is_empty() {
            new_waste_addr.set(name.to_string());
        }
        let _ = js_sys::eval(&format!(
            "if (window.ecoWastePicker) window.ecoWastePicker.setPosition({}, {});",
            lat, lng
        ));
    };

    let suggest_waste_address = move |_| {
        is_reverse_geocoding_waste.set(true);
        let lat = new_waste_lat.get_untracked();
        let lng = new_waste_lng.get_untracked();
        spawn_local(async move {
            let (addr_str, _) = crate::geo::reverse_geocode(lat, lng).await;
            if !addr_str.is_empty() {
                new_waste_addr.set(addr_str);
            }
            is_reverse_geocoding_waste.set(false);
        });
    };

    // Helper functions for Waste Point Modal
    let open_edit_waste_point = move |wp: WastePoint| {
        new_waste_name.set(wp.name.clone());
        new_waste_desc.set(wp.description.clone());
        new_waste_addr.set(wp.address.clone());
        new_waste_lat.set(wp.latitude);
        new_waste_lng.set(wp.longitude);
        new_waste_type.set(wp.waste_type);
        new_waste_fill.set(wp.fill_level);
        new_waste_schedule.set(wp.truck_schedule.clone());
        new_waste_recom.set(wp.recommendations.clone());
        new_waste_img.set(wp.image_url.clone());
        action_error.set(None);
        show_manual_coords_details.set(false);
        editing_waste_target.set(Some(wp));
        show_create_waste_modal.set(true);

        let lat = new_waste_lat.get_untracked();
        let lng = new_waste_lng.get_untracked();
        let _ = js_sys::eval(&format!(
            "if (window.ecoWastePicker) {{ window.ecoWastePicker.init('waste-picker-map', {}, {}); }}",
            lat, lng
        ));
    };

    let open_create_waste_point = move |_| {
        new_waste_name.set("".to_string());
        new_waste_desc.set("".to_string());
        new_waste_addr.set("".to_string());
        new_waste_lat.set(-10.6675);
        new_waste_lng.set(-76.2567);
        new_waste_type.set(WasteType::Recyclable);
        new_waste_fill.set(FillLevel::Low);
        new_waste_schedule.set("Lunes a Sábado 06:00 a 08:30".to_string());
        new_waste_recom.set("".to_string());
        new_waste_img.set("".to_string());
        action_error.set(None);
        show_manual_coords_details.set(false);
        editing_waste_target.set(None);
        show_create_waste_modal.set(true);

        let _ = js_sys::eval("if (window.ecoWastePicker) { window.ecoWastePicker.init('waste-picker-map', -10.6675, -76.2567); }");
    };

    // Action 6: Create or Update Waste Point
    let submit_create_waste_point = move |_| {
        let user = match current_user.get() {
            Some(u) if u.is_authority() => u,
            Some(_) => {
                action_error.set(Some("Acceso denegado: Tu cuenta no tiene rol de autoridad fiscalizadora.".to_string()));
                return;
            }
            None => {
                show_auth_dialog.set(true);
                return;
            }
        };

        let name = new_waste_name.get();
        let desc = new_waste_desc.get();
        let addr = new_waste_addr.get();

        if name.trim().is_empty() {
            action_error.set(Some("Ingresa el nombre del punto de acopio.".to_string()));
            return;
        }

        let payload = CreateWastePointPayload {
            name: name.clone(),
            description: desc.clone(),
            address: addr.clone(),
            latitude: new_waste_lat.get(),
            longitude: new_waste_lng.get(),
            waste_type: new_waste_type.get(),
            fill_level: new_waste_fill.get(),
            truck_schedule: new_waste_schedule.get(),
            image_url: new_waste_img.get(),
            recommendations: new_waste_recom.get(),
        };

        let user_token = user.token.clone();
        let editing_target_opt = editing_waste_target.get();

        is_submitting_action.set(true);
        action_error.set(None);

        spawn_local(async move {
            if let Some(target) = editing_target_opt {
                let pid = target.id_string();
                let req = gloo_net::http::Request::patch(&waste_point_detail_url(&pid))
                    .header("Authorization", &format!("Token {}", user_token));

                match req.json(&payload) {
                    Ok(r) => match r.send().await {
                        Ok(resp) => {
                            if resp.ok() {
                                if let Ok(updated) = resp.json::<WastePoint>().await {
                                    waste_points.update(|list| {
                                        if let Some(item) = list.iter_mut().find(|p| p.id_string() == pid) {
                                            *item = updated;
                                        }
                                    });
                                } else {
                                    waste_points.update(|list| {
                                        if let Some(item) = list.iter_mut().find(|p| p.id_string() == pid) {
                                            item.name = payload.name;
                                            item.description = payload.description;
                                            item.address = payload.address;
                                            item.latitude = payload.latitude;
                                            item.longitude = payload.longitude;
                                            item.waste_type = payload.waste_type;
                                            item.fill_level = payload.fill_level;
                                            item.truck_schedule = payload.truck_schedule;
                                            item.image_url = payload.image_url;
                                            item.recommendations = payload.recommendations;
                                        }
                                    });
                                }
                                show_create_waste_modal.set(false);
                            } else {
                                action_error.set(Some(format!("Error al actualizar el punto ({})", resp.status())));
                            }
                        }
                        Err(e) => action_error.set(Some(format!("Error de red: {}", e))),
                    },
                    Err(e) => action_error.set(Some(format!("Error serializando: {}", e))),
                }
            } else {
                let req = gloo_net::http::Request::post(&waste_points_url())
                    .header("Authorization", &format!("Token {}", user_token));

                match req.json(&payload) {
                    Ok(r) => match r.send().await {
                        Ok(resp) => {
                            if resp.ok() {
                                if let Ok(created) = resp.json::<WastePoint>().await {
                                    waste_points.update(|list| {
                                        let cid = created.id_string();
                                        if let Some(existing) = list.iter_mut().find(|p| p.id_string() == cid) {
                                            *existing = created;
                                        } else {
                                            list.push(created);
                                        }
                                    });
                                }
                                show_create_waste_modal.set(false);
                            } else {
                                action_error.set(Some(format!(
                                    "No se pudo registrar el punto de acopio en el servidor (HTTP {}).",
                                    resp.status()
                                )));
                            }
                        }
                        Err(e) => {
                            action_error.set(Some(format!(
                                "Error de conexión con el servidor al crear el punto de acopio: {}",
                                e
                            )));
                        }
                    },
                    Err(e) => action_error.set(Some(format!("Error serializando: {}", e))),
                }
            }
            is_submitting_action.set(false);
        });
    };

    // Upload resolution photo via JS
    let upload_resolve_photo = move |_| {
        let user = match current_user.get() {
            Some(u) if u.is_authority() => u,
            Some(_) => {
                action_error.set(Some("Acceso denegado: Se requiere rol de autoridad fiscalizadora para subir evidencia.".to_string()));
                return;
            }
            None => {
                show_auth_dialog.set(true);
                return;
            }
        };

        is_uploading_resolve_photo.set(true);
        action_error.set(None);
        let token_arg = user.token.clone();

        spawn_local(async move {
            let js_code = format!(
                "window.ecoUpload.uploadFile('{}', '{}')",
                upload_url(),
                token_arg
            );
            match js_sys::eval(&js_code) {
                Ok(promise_val) => {
                    let promise = js_sys::Promise::from(promise_val);
                    match JsFuture::from(promise).await {
                        Ok(val) => {
                            if let Some(url_str) = val.as_string() {
                                resolve_photo_url.set(Some(url_str));
                            } else {
                                action_error.set(Some("Respuesta inesperada al subir foto.".to_string()));
                            }
                        }
                        Err(e) => {
                            let msg = if let Some(err) = e.dyn_ref::<js_sys::Error>() {
                                err.message().into()
                            } else if let Some(s) = e.as_string() {
                                s
                            } else {
                                "Error al subir la fotografía de resolución.".to_string()
                            };
                            action_error.set(Some(msg));
                        }
                    }
                }
                Err(e) => action_error.set(Some(format!("Error invocando cargador: {:?}", e))),
            }
            is_uploading_resolve_photo.set(false);
        });
    };

    // Gemini AI Management functions
    let fetch_gemini_settings = move || {
        is_loading_settings.set(true);
        gemini_status_msg.set(None);
        gemini_test_result.set(None);

        let token_opt = current_user.get().map(|u| u.token.clone());

        spawn_local(async move {
            let mut req = gloo_net::http::Request::get(&system_settings_url());
            if let Some(token) = token_opt {
                req = req.header("Authorization", &format!("Token {}", token));
            }

            match req.send().await {
                Ok(res) => {
                    if res.ok() {
                        if let Ok(settings) = res.json::<SystemSettingsResponse>().await {
                            gemini_model_input.set(settings.selected_model.clone());
                            gemini_enabled_input.set(settings.gemini_enabled);
                            if let Some(ref sk) = settings.turnstile_sitekey {
                                turnstile_sitekey_input.set(sk.clone());
                            }
                            if let Some(en) = settings.turnstile_enabled {
                                turnstile_enabled_input.set(en);
                            }
                            gemini_settings.set(Some(settings));
                        } else {
                            gemini_status_msg.set(Some((false, "Error al interpretar la configuración del sistema.".to_string())));
                        }
                    } else {
                        gemini_status_msg.set(Some((false, format!("Error del servidor al obtener ajustes ({})", res.status()))));
                    }
                }
                Err(e) => {
                    gemini_status_msg.set(Some((false, format!("Error de conexión: {}", e))));
                }
            }
            is_loading_settings.set(false);
        });
    };

    let open_gemini_modal = move |_| {
        show_gemini_modal.set(true);
        fetch_gemini_settings();
    };

    let test_gemini_connection = move |_| {
        let user = match current_user.get() {
            Some(u) if u.is_authority() => u,
            Some(_) => {
                gemini_status_msg.set(Some((false, "Acceso denegado: Se requiere rol de autoridad fiscalizadora.".to_string())));
                return;
            }
            None => {
                show_auth_dialog.set(true);
                return;
            }
        };

        is_testing_gemini.set(true);
        gemini_test_result.set(None);
        gemini_status_msg.set(None);

        let key = gemini_key_input.get().trim().to_string();
        let model = gemini_model_input.get();
        let token = user.token.clone();

        spawn_local(async move {
            let mut payload = serde_json::Map::new();
            if !key.is_empty() {
                payload.insert("gemini_api_key".to_string(), serde_json::Value::String(key));
            }
            payload.insert("gemini_model".to_string(), serde_json::Value::String(model));

            let res = gloo_net::http::Request::post(&test_gemini_url())
                .header("Authorization", &format!("Token {}", token))
                .json(&serde_json::Value::Object(payload));

            match res {
                Ok(req) => match req.send().await {
                    Ok(resp) => {
                        let status_code = resp.status();
                        if let Ok(diag) = resp.json::<TestGeminiResponse>().await {
                            if diag.success {
                                gemini_status_msg.set(Some((true, format!("¡Handshake exitoso! Latencia: {}ms · Estado: {}", diag.latency_ms.unwrap_or(0), diag.status.as_deref().unwrap_or("OPERATIVO")))));
                            } else {
                                let err_msg = diag.error.as_deref().or(diag.message.as_deref()).unwrap_or("Fallo en la prueba de conexión con Gemini");
                                gemini_status_msg.set(Some((false, err_msg.to_string())));
                            }
                            gemini_test_result.set(Some(diag));
                        } else {
                            gemini_status_msg.set(Some((false, format!("Error HTTP {} al probar conexión con Gemini.", status_code))));
                        }
                    }
                    Err(e) => {
                        gemini_status_msg.set(Some((false, format!("Error de red al contactar Gemini: {}", e))));
                    }
                },
                Err(e) => {
                    gemini_status_msg.set(Some((false, format!("Error construyendo petición: {}", e))));
                }
            }
            is_testing_gemini.set(false);
        });
    };

    let save_gemini_settings = move |_| {
        let user = match current_user.get() {
            Some(u) if u.is_authority() => u,
            Some(_) => {
                gemini_status_msg.set(Some((false, "Acceso denegado: Se requiere rol de autoridad fiscalizadora.".to_string())));
                return;
            }
            None => {
                show_auth_dialog.set(true);
                return;
            }
        };

        let key = gemini_key_input.get().trim().to_string();
        let model = gemini_model_input.get();
        let enabled = gemini_enabled_input.get();
        let has_existing_key = gemini_settings.get().map(|s| s.has_gemini_key).unwrap_or(false);

        let ts_sitekey = turnstile_sitekey_input.get().trim().to_string();
        let ts_secret = turnstile_secret_input.get().trim().to_string();
        let ts_enabled = turnstile_enabled_input.get();

        if key.is_empty() && !has_existing_key && enabled {
            gemini_status_msg.set(Some((false, "Ingresa una clave de API antes de activar Gemini.".to_string())));
            return;
        }

        is_saving_settings.set(true);
        gemini_status_msg.set(None);
        let token = user.token.clone();

        spawn_local(async move {
            let mut payload = serde_json::Map::new();
            if !key.is_empty() {
                payload.insert("gemini_api_key".to_string(), serde_json::Value::String(key.clone()));
            }
            payload.insert("gemini_model".to_string(), serde_json::Value::String(model.clone()));
            payload.insert("gemini_enabled".to_string(), serde_json::Value::Bool(enabled));

            if !ts_sitekey.is_empty() {
                payload.insert("turnstile_sitekey".to_string(), serde_json::Value::String(ts_sitekey));
            }
            if !ts_secret.is_empty() {
                payload.insert("turnstile_secret_key".to_string(), serde_json::Value::String(ts_secret));
            }
            payload.insert("turnstile_enabled".to_string(), serde_json::Value::Bool(ts_enabled));

            let res = gloo_net::http::Request::post(&system_settings_url())
                .header("Authorization", &format!("Token {}", token))
                .json(&serde_json::Value::Object(payload));

            match res {
                Ok(req) => match req.send().await {
                    Ok(resp) => {
                        if resp.ok() {
                            gemini_key_input.set("".to_string());
                            turnstile_secret_input.set("".to_string());
                            let success_text = if enabled {
                                format!("¡Configuración guardada! Gemini ({}) y Cloudflare Turnstile actualizados.", model)
                            } else {
                                "¡Configuración guardada! Modo manual activado y ajustes de Turnstile guardados.".to_string()
                            };
                            gemini_status_msg.set(Some((true, success_text)));

                            if let Ok(updated) = resp.json::<SystemSettingsResponse>().await {
                                gemini_settings.set(Some(updated));
                            }
                        } else {
                            gemini_status_msg.set(Some((false, format!("Error HTTP {} al guardar ajustes.", resp.status()))));
                        }
                    }
                    Err(e) => {
                        gemini_status_msg.set(Some((false, format!("Error de conexión al guardar: {}", e))));
                    }
                },
                Err(e) => {
                    gemini_status_msg.set(Some((false, format!("Error serializando: {}", e))));
                }
            }
            is_saving_settings.set(false);
        });
    };

    view! {
        {move || show_dashboard.get().then(|| {
            let is_authority = current_user.get().map(|u| u.is_authority()).unwrap_or(false);
            if !is_authority {
                view! {
                    <div class="fixed inset-0 z-50 bg-slate-900/60 backdrop-blur-md flex items-center justify-center p-4 animate-in fade-in duration-200">
                        <div class="bg-white max-w-md w-full rounded-2xl border border-red-200 shadow-2xl p-6 sm:p-8 text-center space-y-4">
                            <div class="w-16 h-16 mx-auto rounded-2xl bg-red-50 text-red-600 border border-red-100 flex items-center justify-center text-3xl shadow-xs">
                                "🛑"
                            </div>
                            <div class="space-y-1.5">
                                <span class="text-[10px] font-bold uppercase tracking-wider px-2.5 py-1 rounded-full bg-red-100 text-red-800 border border-red-200">
                                    "Acceso Restringido"
                                </span>
                                <h2 class="text-lg font-bold text-slate-900 pt-1">
                                    "Área Exclusiva para Autoridades"
                                </h2>
                                <p class="text-xs text-slate-600 leading-relaxed">
                                    {move || {
                                        if let Some(user) = current_user.get() {
                                            format!("Tu cuenta actual ('{}') no tiene asignados roles de autoridad ni permisos de fiscalización ambiental. Este módulo está restringido exclusivamente a inspectores y funcionarios municipales acreditados.", user.username)
                                        } else {
                                            "Debes iniciar sesión con una cuenta autorizada con rol de fiscalizador o inspector municipal para acceder a este panel de control.".to_string()
                                        }
                                    }}
                                </p>
                            </div>
                            <div class="pt-2 flex flex-col sm:flex-row gap-2.5 justify-center">
                                <button
                                    class="w-full py-2.5 px-4 rounded-xl text-xs font-bold text-slate-700 bg-slate-100 hover:bg-slate-200 border border-slate-200 transition"
                                    on:click=move |_| show_dashboard.set(false)
                                >
                                    "Volver al Mapa"
                                </button>
                                {move || if current_user.get().is_none() {
                                    view! {
                                        <button
                                            class="w-full py-2.5 px-4 rounded-xl text-xs font-bold text-white bg-slate-900 hover:bg-slate-800 transition"
                                            on:click=move |_| {
                                                show_dashboard.set(false);
                                                show_auth_dialog.set(true);
                                            }
                                        >
                                            "Iniciar Sesión"
                                        </button>
                                    }.into_view()
                                } else {
                                    view! {
                                        <button
                                            class="w-full py-2.5 px-4 rounded-xl text-xs font-bold text-red-700 bg-red-50 hover:bg-red-100 border border-red-200 transition"
                                            on:click=move |_| {
                                                crate::storage::logout_and_clear_session(current_user);
                                                show_dashboard.set(false);
                                                show_auth_dialog.set(true);
                                            }
                                        >
                                            "Cambiar de Cuenta"
                                        </button>
                                    }.into_view()
                                }}
                            </div>
                        </div>
                    </div>
                }.into_view()
            } else {
                view! {
                    <div class="fixed inset-0 z-40 bg-slate-50 flex flex-col overflow-y-auto animate-in fade-in duration-200">
                        // Top Bar
                        <header class="sticky top-0 z-30 bg-white/95 backdrop-blur-md border-b border-slate-200 px-4 sm:px-8 py-3.5 flex items-center justify-between gap-4">
                            <div class="flex items-center space-x-3">
                                <div class="w-10 h-10 rounded-xl bg-slate-900 text-white flex items-center justify-center shrink-0 shadow-xs">
                                    <svg class="w-5 h-5 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                        <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
                                    </svg>
                                </div>
                                <div>
                                    <h1 class="text-base sm:text-lg font-bold text-slate-900 leading-tight">
                                        "Panel de Control de Autoridades"
                                    </h1>
                                    <p class="text-xs text-slate-500 font-medium">
                                        "Fiscalización ambiental, semáforo SLA y gestión municipal · Cerro de Pasco"
                                    </p>
                                </div>
                            </div>

                            <div class="flex items-center space-x-2">
                                {move || current_user.get().map(|user| {
                                    let username = user.username.clone();
                                    let role = user.role_label();
                                    view! {
                                        <div class="hidden sm:flex items-center space-x-2 px-3 py-1.5 rounded-xl bg-slate-100 border border-slate-200/80 text-xs">
                                            <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
                                            <span class="font-bold text-slate-800">{username}</span>
                                            <span class="text-emerald-700 font-bold bg-emerald-100 px-2 py-0.5 rounded-full text-[10px]">
                                                {role}
                                            </span>
                                        </div>
                                    }
                                })}

                                <button
                                    class="px-3.5 py-2 text-xs font-semibold bg-sky-50 text-sky-700 hover:bg-sky-100 border border-sky-200 rounded-xl shadow-xs transition flex items-center space-x-1.5 cursor-pointer"
                                    title="Configurar IA Gemini y Seguridad Turnstile"
                                    on:click=open_gemini_modal
                                >
                                    <svg class="w-4 h-4 text-sky-600 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                        <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/>
                                        <circle cx="12" cy="12" r="3"/>
                                    </svg>
                                    <span class="hidden sm:inline">"Ajustes del Sistema"</span>
                                </button>

                            <button
                                class="px-4 py-2 text-xs font-semibold bg-slate-900 hover:bg-slate-800 text-white rounded-xl shadow-xs transition flex items-center space-x-1.5"
                                on:click=move |_| show_dashboard.set(false)
                            >
                                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <polygon points="3 6 9 3 15 6 21 3 21 18 15 21 9 18 3 21"/>
                                    <line x1="9" y1="3" x2="9" y2="18"/>
                                    <line x1="15" y1="6" x2="15" y2="21"/>
                                </svg>
                                <span>"Volver al Mapa"</span>
                            </button>
                        </div>
                    </header>

                    // Dashboard Tabs Selector
                    <div class="bg-white border-b border-slate-200/80 px-4 sm:px-8 py-2.5">
                        <div class="max-w-6xl mx-auto flex items-center space-x-2">
                            <button
                                class=move || {
                                    if active_tab.get() == 0 {
                                        "px-4 py-2 rounded-xl text-xs font-bold bg-slate-900 text-white shadow-xs flex items-center space-x-2 transition"
                                    } else {
                                        "px-4 py-2 rounded-xl text-xs font-semibold text-slate-600 hover:bg-slate-100 flex items-center space-x-2 transition"
                                    }
                                }
                                on:click=move |_| active_tab.set(0)
                            >
                                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
                                    <line x1="12" y1="9" x2="12" y2="13"/>
                                    <line x1="12" y1="17" x2="12.01" y2="17"/>
                                </svg>
                                <span>"Denuncias Ambientales"</span>
                                <span class="px-1.5 py-0.5 rounded-full text-[10px] bg-white/20 text-white font-mono">
                                    {total_count}
                                </span>
                            </button>

                            <button
                                class=move || {
                                    if active_tab.get() == 1 {
                                        "px-4 py-2 rounded-xl text-xs font-bold bg-slate-900 text-white shadow-xs flex items-center space-x-2 transition"
                                    } else {
                                        "px-4 py-2 rounded-xl text-xs font-semibold text-slate-600 hover:bg-slate-100 flex items-center space-x-2 transition"
                                    }
                                }
                                on:click=move |_| active_tab.set(1)
                            >
                                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <path d="M7 19H4.815a1.83 1.83 0 0 1-1.57-.881 1.785 1.785 0 0 1-.004-1.784L7.196 9.5"/>
                                    <path d="M11 19h8.2a1.8 1.8 0 0 0 1.573-.896 1.79 1.79 0 0 0 0-1.792L16.8 9.5"/>
                                    <path d="m15.5 13-3.5 6-3.5-6"/>
                                    <path d="m2 9 3-5 3 5"/>
                                </svg>
                                <span>"Puntos de Acopio & Contenedores"</span>
                                <span class="px-1.5 py-0.5 rounded-full text-[10px] bg-slate-200 text-slate-700 font-mono">
                                    {move || waste_points.get().len()}
                                </span>
                            </button>

                            <button
                                class=move || {
                                    if active_tab.get() == 2 {
                                        "px-4 py-2 rounded-xl text-xs font-bold bg-slate-900 text-white shadow-xs flex items-center space-x-2 transition"
                                    } else {
                                        "px-4 py-2 rounded-xl text-xs font-semibold text-slate-600 hover:bg-slate-100 flex items-center space-x-2 transition"
                                    }
                                }
                                on:click=move |_| active_tab.set(2)
                            >
                                <span>"🚛"</span>
                                <span>"Rutas y Flota"</span>
                                <span class="px-1.5 py-0.5 rounded-full text-[10px] bg-emerald-100 text-emerald-800 font-mono">
                                    {move || truck_runs.with(|r| r.iter().filter(|x| x.status == "active").count())}
                                </span>
                            </button>
                        </div>
                    </div>

                    // Main Container
                    <main class="flex-1 max-w-6xl w-full mx-auto px-4 sm:px-8 py-6 space-y-6">
                        {move || if active_tab.get() == 2 {
                            view! {
                                <FleetRoutesTab
                                    truck_runs=truck_runs
                                    waste_points=waste_points
                                    current_user=current_user
                                    show_dashboard=show_dashboard
                                />
                            }.into_view()
                        } else if active_tab.get() == 0 {
                            view! {
                                <div class="space-y-6 animate-in fade-in duration-150">
                                    // 5 KPI Cards Row
                                    <div class="grid grid-cols-2 sm:grid-cols-5 gap-3">
                                        <div class="p-3.5 rounded-2xl bg-white border border-slate-200 shadow-xs">
                                            <p class="text-xs font-medium text-slate-500">"Total Denuncias"</p>
                                            <h3 class="text-xl font-bold text-slate-900 font-mono mt-0.5">{total_count}</h3>
                                            <p class="text-[10px] text-slate-400">"Registradas en Pasco"</p>
                                        </div>
                                        <div class="p-3.5 rounded-2xl bg-white border border-amber-200/80 shadow-xs">
                                            <p class="text-xs font-medium text-amber-700">"Pendientes"</p>
                                            <h3 class="text-xl font-bold text-amber-900 font-mono mt-0.5">{pending_count}</h3>
                                            <p class="text-[10px] text-amber-600/80">"Por asignar/revisar"</p>
                                        </div>
                                        <div class="p-3.5 rounded-2xl bg-white border border-purple-200/80 shadow-xs">
                                            <p class="text-xs font-medium text-purple-700">"En Inspección"</p>
                                            <h3 class="text-xl font-bold text-purple-900 font-mono mt-0.5">{verified_count}</h3>
                                            <p class="text-[10px] text-purple-600/80">"Con cuadrilla en campo"</p>
                                        </div>
                                        <div class="p-3.5 rounded-2xl bg-white border border-emerald-200/80 shadow-xs">
                                            <p class="text-xs font-medium text-emerald-700">"Resueltos"</p>
                                            <h3 class="text-xl font-bold text-emerald-900 font-mono mt-0.5">{solved_count}</h3>
                                            <p class="text-[10px] text-emerald-600/80">"Con informe oficial"</p>
                                        </div>
                                        <div class="p-3.5 rounded-2xl bg-white border border-red-200/80 shadow-xs col-span-2 sm:col-span-1">
                                            <p class="text-xs font-medium text-red-700">"SLA Urgente"</p>
                                            <h3 class="text-xl font-bold text-red-900 font-mono mt-0.5">{urgent_count}</h3>
                                            <p class="text-[10px] text-red-600/80">">24h sin solución"</p>
                                        </div>
                                    </div>

                                    // Analytics Charts Row (Category, Severity, Municipality Load)
                                    <div class="space-y-3">
                                        <div class="flex items-center justify-between">
                                            <span class="text-xs font-bold text-slate-800 uppercase tracking-wider flex items-center space-x-1.5">
                                                <span>"📊"</span>
                                                <span>"Métricas y Analítica de Fiscalización"</span>
                                            </span>
                                            <button
                                                class="text-[11px] font-semibold text-slate-500 hover:text-slate-800 transition flex items-center space-x-1 cursor-pointer"
                                                on:click=move |_| show_analytics.update(|v| *v = !*v)
                                            >
                                                {move || if show_analytics.get() { "Ocultar Gráficos ▲" } else { "Mostrar Gráficos ▼" }}
                                            </button>
                                        </div>

                                        {move || show_analytics.get().then(|| {
                                            let all = alerts.get();
                                            let total = all.len();
                                            let total_f = if total == 0 { 1.0 } else { total as f64 };

                                            let mineria = all.iter().filter(|a| a.category == EcoCategory::Mineria).count();
                                            let basura = all.iter().filter(|a| a.category == EcoCategory::Basura).count();
                                            let agua = all.iter().filter(|a| a.category == EcoCategory::Agua).count();
                                            let aire = all.iter().filter(|a| a.category == EcoCategory::Aire).count();

                                            let mineria_pct = (mineria as f64 * 100.0) / total_f;
                                            let basura_pct = (basura as f64 * 100.0) / total_f;
                                            let agua_pct = (agua as f64 * 100.0) / total_f;
                                            let aire_pct = (aire as f64 * 100.0) / total_f;

                                            let critico = all.iter().filter(|a| a.severity == EcoSeverity::Critico).count();
                                            let medio = all.iter().filter(|a| a.severity == EcoSeverity::Medio).count();
                                            let bajo = all.iter().filter(|a| a.severity == EcoSeverity::Bajo).count();

                                            let critico_pct = (critico as f64 * 100.0) / total_f;
                                            let medio_pct = (medio as f64 * 100.0) / total_f;
                                            let bajo_pct = (bajo as f64 * 100.0) / total_f;

                                            let chaupimarca = all.iter().filter(|a| a.district == EcoDistrict::Chaupimarca).count();
                                            let yanacancha = all.iter().filter(|a| a.district == EcoDistrict::Yanacancha).count();
                                            let simon = all.iter().filter(|a| a.district == EcoDistrict::SimonBolivar).count();

                                            let chaupimarca_pct = (chaupimarca as f64 * 100.0) / total_f;
                                            let yanacancha_pct = (yanacancha as f64 * 100.0) / total_f;
                                            let simon_pct = (simon as f64 * 100.0) / total_f;

                                            view! {
                                                <div class="grid grid-cols-1 md:grid-cols-3 gap-3.5 animate-in fade-in duration-200">
                                                    // Card 1: Category Distribution
                                                    <div class="p-4 rounded-2xl bg-white border border-slate-200 shadow-xs space-y-3">
                                                        <h4 class="text-xs font-bold text-slate-800 uppercase tracking-wider">
                                                            "Distribución por Categoría"
                                                        </h4>
                                                        <div class="flex items-center space-x-3">
                                                            <div class="relative w-24 h-24 shrink-0">
                                                                <svg viewBox="0 0 42 42" class="w-full h-full transform -rotate-90">
                                                                    <circle cx="21" cy="21" r="15.91549430918954" fill="transparent" stroke="#f1f5f9" stroke-width="6.5"></circle>
                                                                    <circle cx="21" cy="21" r="15.91549430918954" fill="transparent" stroke="#d97706" stroke-width="6.5" stroke-dasharray=format!("{} {}", mineria_pct, 100.0 - mineria_pct) stroke-dashoffset="0"></circle>
                                                                    <circle cx="21" cy="21" r="15.91549430918954" fill="transparent" stroke="#059669" stroke-width="6.5" stroke-dasharray=format!("{} {}", basura_pct, 100.0 - basura_pct) stroke-dashoffset=format!("-{}", mineria_pct)></circle>
                                                                    <circle cx="21" cy="21" r="15.91549430918954" fill="transparent" stroke="#0284c7" stroke-width="6.5" stroke-dasharray=format!("{} {}", agua_pct, 100.0 - agua_pct) stroke-dashoffset=format!("-{}", mineria_pct + basura_pct)></circle>
                                                                    <circle cx="21" cy="21" r="15.91549430918954" fill="transparent" stroke="#7c3aed" stroke-width="6.5" stroke-dasharray=format!("{} {}", aire_pct, 100.0 - aire_pct) stroke-dashoffset=format!("-{}", mineria_pct + basura_pct + agua_pct)></circle>
                                                                </svg>
                                                                <div class="absolute inset-0 flex flex-col items-center justify-center pointer-events-none">
                                                                    <span class="text-xs font-black text-slate-900 leading-tight">{total}</span>
                                                                    <span class="text-[8px] text-slate-400 uppercase font-semibold">"Total"</span>
                                                                </div>
                                                            </div>

                                                            <div class="flex-1 space-y-1 text-[11px] min-w-0">
                                                                <div class="flex items-center justify-between text-slate-700">
                                                                    <span class="flex items-center space-x-1.5 truncate">
                                                                        <span class="w-2 h-2 rounded-full bg-amber-600 shrink-0"></span>
                                                                        <span>"Minería"</span>
                                                                    </span>
                                                                    <span class="font-bold text-slate-900 font-mono">{format!("{} ({:.0}%)", mineria, mineria_pct)}</span>
                                                                </div>
                                                                <div class="flex items-center justify-between text-slate-700">
                                                                    <span class="flex items-center space-x-1.5 truncate">
                                                                        <span class="w-2 h-2 rounded-full bg-emerald-600 shrink-0"></span>
                                                                        <span>"Residuos"</span>
                                                                    </span>
                                                                    <span class="font-bold text-slate-900 font-mono">{format!("{} ({:.0}%)", basura, basura_pct)}</span>
                                                                </div>
                                                                <div class="flex items-center justify-between text-slate-700">
                                                                    <span class="flex items-center space-x-1.5 truncate">
                                                                        <span class="w-2 h-2 rounded-full bg-sky-600 shrink-0"></span>
                                                                        <span>"Agua"</span>
                                                                    </span>
                                                                    <span class="font-bold text-slate-900 font-mono">{format!("{} ({:.0}%)", agua, agua_pct)}</span>
                                                                </div>
                                                                <div class="flex items-center justify-between text-slate-700">
                                                                    <span class="flex items-center space-x-1.5 truncate">
                                                                        <span class="w-2 h-2 rounded-full bg-purple-600 shrink-0"></span>
                                                                        <span>"Aire"</span>
                                                                    </span>
                                                                    <span class="font-bold text-slate-900 font-mono">{format!("{} ({:.0}%)", aire, aire_pct)}</span>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    </div>

                                                    // Card 2: Severity Distribution
                                                    <div class="p-4 rounded-2xl bg-white border border-slate-200 shadow-xs space-y-3">
                                                        <h4 class="text-xs font-bold text-slate-800 uppercase tracking-wider">
                                                            "Casos por Nivel de Gravedad"
                                                        </h4>
                                                        <div class="space-y-2.5 pt-1 text-xs">
                                                            <div>
                                                                <div class="flex items-center justify-between text-[11px] mb-1">
                                                                    <span class="font-bold text-red-700 flex items-center space-x-1">
                                                                        <span>"💀"</span>
                                                                        <span>"Crítico"</span>
                                                                    </span>
                                                                    <span class="font-bold text-slate-900 font-mono">{format!("{} ({:.0}%)", critico, critico_pct)}</span>
                                                                </div>
                                                                <div class="w-full bg-slate-100 rounded-full h-2 overflow-hidden">
                                                                    <div class="bg-red-500 h-2 rounded-full transition-all duration-500" style=format!("width: {}%", critico_pct)></div>
                                                                </div>
                                                            </div>

                                                            <div>
                                                                <div class="flex items-center justify-between text-[11px] mb-1">
                                                                    <span class="font-bold text-amber-700 flex items-center space-x-1">
                                                                        <span>"⚠️"</span>
                                                                        <span>"Medio"</span>
                                                                    </span>
                                                                    <span class="font-bold text-slate-900 font-mono">{format!("{} ({:.0}%)", medio, medio_pct)}</span>
                                                                </div>
                                                                <div class="w-full bg-slate-100 rounded-full h-2 overflow-hidden">
                                                                    <div class="bg-amber-500 h-2 rounded-full transition-all duration-500" style=format!("width: {}%", medio_pct)></div>
                                                                </div>
                                                            </div>

                                                            <div>
                                                                <div class="flex items-center justify-between text-[11px] mb-1">
                                                                    <span class="font-bold text-emerald-700 flex items-center space-x-1">
                                                                        <span>"🍃"</span>
                                                                        <span>"Bajo"</span>
                                                                    </span>
                                                                    <span class="font-bold text-slate-900 font-mono">{format!("{} ({:.0}%)", bajo, bajo_pct)}</span>
                                                                </div>
                                                                <div class="w-full bg-slate-100 rounded-full h-2 overflow-hidden">
                                                                    <div class="bg-emerald-500 h-2 rounded-full transition-all duration-500" style=format!("width: {}%", bajo_pct)></div>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    </div>

                                                    // Card 3: Municipality Load by District
                                                    <div class="p-4 rounded-2xl bg-white border border-slate-200 shadow-xs space-y-3">
                                                        <h4 class="text-xs font-bold text-slate-800 uppercase tracking-wider">
                                                            "Carga por Jurisdicción Municipal"
                                                        </h4>
                                                        <div class="space-y-2.5 pt-1 text-xs">
                                                            <div>
                                                                <div class="flex items-center justify-between text-[11px] mb-1">
                                                                    <span class="font-semibold text-slate-700 truncate" title="Municipalidad Provincial de Pasco">"Muni. Chaupimarca"</span>
                                                                    <span class="font-bold text-slate-900 font-mono ml-2 shrink-0">{format!("{} ({:.0}%)", chaupimarca, chaupimarca_pct)}</span>
                                                                </div>
                                                                <div class="w-full bg-slate-100 rounded-full h-2 overflow-hidden">
                                                                    <div class="bg-indigo-500 h-2 rounded-full transition-all duration-500" style=format!("width: {}%", chaupimarca_pct)></div>
                                                                </div>
                                                            </div>

                                                            <div>
                                                                <div class="flex items-center justify-between text-[11px] mb-1">
                                                                    <span class="font-semibold text-slate-700 truncate" title="Municipalidad Distrital de Yanacancha">"Muni. Yanacancha"</span>
                                                                    <span class="font-bold text-slate-900 font-mono ml-2 shrink-0">{format!("{} ({:.0}%)", yanacancha, yanacancha_pct)}</span>
                                                                </div>
                                                                <div class="w-full bg-slate-100 rounded-full h-2 overflow-hidden">
                                                                    <div class="bg-sky-500 h-2 rounded-full transition-all duration-500" style=format!("width: {}%", yanacancha_pct)></div>
                                                                </div>
                                                            </div>

                                                            <div>
                                                                <div class="flex items-center justify-between text-[11px] mb-1">
                                                                    <span class="font-semibold text-slate-700 truncate" title="Municipalidad Distrital de Simón Bolívar">"Muni. Simón Bolívar"</span>
                                                                    <span class="font-bold text-slate-900 font-mono ml-2 shrink-0">{format!("{} ({:.0}%)", simon, simon_pct)}</span>
                                                                </div>
                                                                <div class="w-full bg-slate-100 rounded-full h-2 overflow-hidden">
                                                                    <div class="bg-purple-500 h-2 rounded-full transition-all duration-500" style=format!("width: {}%", simon_pct)></div>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                            }.into_view()
                                        })}
                                    </div>

                                    // Filter bar
                                    <div class="p-4 bg-white rounded-2xl border border-slate-200 shadow-xs space-y-3">
                                        <div class="flex flex-col sm:flex-row gap-3 items-center justify-between">
                                            <div class="relative w-full sm:w-72">
                                                <svg class="w-4 h-4 text-slate-400 absolute left-3 top-1/2 transform -translate-y-1/2" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                    <circle cx="11" cy="11" r="8"></circle>
                                                    <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                                                </svg>
                                                <input
                                                    type="text"
                                                    placeholder="Buscar denuncia, zona o calle..."
                                                    class="w-full pl-9 pr-3 py-1.5 text-xs bg-slate-50 border border-slate-200 rounded-xl focus:bg-white focus:outline-hidden focus:ring-2 focus:ring-slate-900 transition"
                                                    prop:value=move || search_query.get()
                                                    on:input=move |e| search_query.set(event_target_value(&e))
                                                />
                                            </div>

                                            // Status Filter Chips
                                            <div class="flex items-center space-x-1.5 overflow-x-auto w-full sm:w-auto text-xs">
                                                <button
                                                    class=move || {
                                                        if filter_status.get() == "todos" { "px-2.5 py-1 rounded-lg bg-slate-900 text-white font-semibold" } else { "px-2.5 py-1 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium" }
                                                    }
                                                    on:click=move |_| filter_status.set("todos".to_string())
                                                >
                                                    "Todos"
                                                </button>
                                                <button
                                                    class=move || {
                                                        if filter_status.get() == "pending" { "px-2.5 py-1 rounded-lg bg-amber-700 text-white font-semibold" } else { "px-2.5 py-1 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium" }
                                                    }
                                                    on:click=move |_| filter_status.set("pending".to_string())
                                                >
                                                    "Pendientes"
                                                </button>
                                                <button
                                                    class=move || {
                                                        if filter_status.get() == "verified" { "px-2.5 py-1 rounded-lg bg-purple-700 text-white font-semibold" } else { "px-2.5 py-1 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium" }
                                                    }
                                                    on:click=move |_| filter_status.set("verified".to_string())
                                                >
                                                    "En Inspección"
                                                </button>
                                                <button
                                                    class=move || {
                                                        if filter_status.get() == "solved" { "px-2.5 py-1 rounded-lg bg-emerald-700 text-white font-semibold" } else { "px-2.5 py-1 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium" }
                                                    }
                                                    on:click=move |_| filter_status.set("solved".to_string())
                                                >
                                                    "Resueltos"
                                                </button>
                                                <button
                                                    class=move || {
                                                        if filter_status.get() == "dismissed" { "px-2.5 py-1 rounded-lg bg-slate-700 text-white font-semibold" } else { "px-2.5 py-1 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium" }
                                                    }
                                                    on:click=move |_| filter_status.set("dismissed".to_string())
                                                >
                                                    "Descartados"
                                                </button>
                                            </div>
                                        </div>

                                        // Second row: District chips and SLA Urgent toggle
                                        <div class="flex flex-wrap items-center justify-between gap-2 pt-2 border-t border-slate-100 text-xs">
                                            <div class="flex items-center space-x-1.5">
                                                <span class="text-[10px] font-bold text-slate-400 uppercase tracking-wider mr-1">"Distrito:"</span>
                                                <button
                                                    class=move || {
                                                        if filter_district.get().is_none() { "px-2 py-0.5 rounded-lg bg-slate-900 text-white font-semibold" } else { "px-2 py-0.5 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium" }
                                                    }
                                                    on:click=move |_| filter_district.set(None)
                                                >
                                                    "Todos"
                                                </button>
                                                {EcoDistrict::all().iter().map(|&d| {
                                                    let is_sel = move || filter_district.get() == Some(d);
                                                    view! {
                                                        <button
                                                            class=move || {
                                                                if is_sel() { "px-2 py-0.5 rounded-lg bg-slate-900 text-white font-semibold" } else { "px-2 py-0.5 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium" }
                                                            }
                                                            on:click=move |_| filter_district.set(Some(d))
                                                        >
                                                            {d.display_name()}
                                                        </button>
                                                    }
                                                }).collect_view()}
                                            </div>

                                            // SLA Urgent Checkbox toggle
                                            <button
                                                class=move || {
                                                    if filter_urgent_sla.get() {
                                                        "px-2.5 py-1 rounded-lg bg-red-600 text-white font-semibold shadow-xs flex items-center space-x-1.5 transition"
                                                    } else {
                                                        "px-2.5 py-1 rounded-lg bg-red-50 text-red-700 hover:bg-red-100 border border-red-200 font-medium flex items-center space-x-1.5 transition"
                                                    }
                                                }
                                                on:click=move |_| filter_urgent_sla.update(|v| *v = !*v)
                                            >
                                                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                    <circle cx="12" cy="12" r="10"></circle>
                                                    <polyline points="12 6 12 12 16 14"></polyline>
                                                </svg>
                                                <span>"Filtrar SLA Urgente / Vencido"</span>
                                            </button>
                                        </div>
                                    </div>

                                    // Alerts Table
                                    <div class="bg-white rounded-2xl border border-slate-200 shadow-xs overflow-hidden">
                                        <div class="overflow-x-auto">
                                            <table class="w-full text-left text-xs">
                                                <thead class="bg-slate-50 border-b border-slate-200 text-slate-500 uppercase tracking-wider text-[10px]">
                                                    <tr>
                                                        <th class="py-3 px-4 font-bold">"Incidente"</th>
                                                        <th class="py-3 px-3 font-bold">"Severidad"</th>
                                                        <th class="py-3 px-3 font-bold">"Jurisdicción"</th>
                                                        <th class="py-3 px-3 font-bold">"Semáforo SLA"</th>
                                                        <th class="py-3 px-3 font-bold">"Estado"</th>
                                                        <th class="py-3 px-4 font-bold text-right">"Acciones de Autoridad"</th>
                                                    </tr>
                                                </thead>
                                                <tbody class="divide-y divide-slate-100">
                                                    {move || {
                                                        let list = displayed_alerts.get();
                                                        if list.is_empty() {
                                                            view! {
                                                                <tr>
                                                                    <td colspan="6" class="py-12 text-center text-slate-400">
                                                                        "No se encontraron denuncias que coincidan con los filtros."
                                                                    </td>
                                                                </tr>
                                                            }.into_view()
                                                        } else {
                                                            list.into_iter().map(|a| {
                                                                let alert_for_resolve = a.clone();
                                                                let alert_for_dismiss = a.clone();
                                                                let alert_for_transfer = a.clone();
                                                                let (_, _, sla_text, sla_class) = get_alert_sla(&a);

                                                                view! {
                                                                    <tr class="hover:bg-slate-50/70 transition">
                                                                        // Title & Address
                                                                        <td class="py-3.5 px-4 max-w-xs">
                                                                            <p class="font-bold text-slate-900 leading-snug truncate">
                                                                                {a.title.clone()}
                                                                            </p>
                                                                            <button
                                                                                type="button"
                                                                                class="text-[11px] text-slate-500 hover:text-emerald-700 flex items-center space-x-1 truncate mt-0.5 text-left group cursor-pointer transition"
                                                                                title="Centrar esta denuncia en el mapa interactivo"
                                                                                on:click={
                                                                                    let a_for_map = a.clone();
                                                                                    move |_| {
                                                                                        selected_alert_id.set(Some(a_for_map.id.clone()));
                                                                                        show_dashboard.set(false);
                                                                                        let _ = js_sys::eval(&format!(
                                                                                            "if (window.ecoMap && window.ecoMap.flyTo) window.ecoMap.flyTo({}, {}, 16.5);",
                                                                                            a_for_map.latitude, a_for_map.longitude
                                                                                        ));
                                                                                    }
                                                                                }
                                                                            >
                                                                                <i class="fa-solid fa-location-dot text-slate-400 group-hover:text-emerald-600 text-[10px]"></i>
                                                                                <span class="truncate">{a.address.clone()}</span>
                                                                            </button>
                                                                            <span class="text-[10px] text-slate-400 font-mono mt-0.5 inline-block">
                                                                                {a.created_at.clone()}
                                                                            </span>
                                                                        </td>

                                                                        // Severity
                                                                        <td class="py-3.5 px-3">
                                                                            {match a.severity {
                                                                                EcoSeverity::Critico => view! {
                                                                                    <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-red-600 text-white text-[10px] font-bold shadow-2xs">
                                                                                        <svg class="w-2.5 h-2.5 text-white shrink-0" viewBox="0 0 20 20" fill="currentColor">
                                                                                            <path fill-rule="evenodd" d="M8.485 2.495c.673-1.167 2.357-1.167 3.03 0l6.28 10.875c.673 1.167-.17 2.625-1.516 2.625H3.72c-2.309 0-3.752-2.5-2.598-4.5L9.4 3.003ZM10 5a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-1.5 0v-3.5A.75.75 0 0 1 10 5Zm0 9a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z" clip-rule="evenodd" />
                                                                                        </svg>
                                                                                        <span>"Crítico"</span>
                                                                                    </span>
                                                                                }.into_view(),
                                                                                EcoSeverity::Medio => view! {
                                                                                    <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-amber-500 text-white text-[10px] font-bold shadow-2xs">
                                                                                        <svg class="w-2.5 h-2.5 text-white shrink-0" viewBox="0 0 20 20" fill="currentColor">
                                                                                            <path fill-rule="evenodd" d="M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0Zm-8-5a.75.75 0 0 1 .75.75v4.5a.75.75 0 0 1-1.5 0v-4.5A.75.75 0 0 1 10 5Zm0 10a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z" clip-rule="evenodd" />
                                                                                        </svg>
                                                                                        <span>"Medio"</span>
                                                                                    </span>
                                                                                }.into_view(),
                                                                                EcoSeverity::Bajo => view! {
                                                                                    <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-emerald-600 text-white text-[10px] font-bold shadow-2xs">
                                                                                        <svg class="w-2.5 h-2.5 text-white shrink-0" viewBox="0 0 20 20" fill="currentColor">
                                                                                            <path fill-rule="evenodd" d="M10 18a8 8 0 1 0 0-16 8 8 0 0 0 0 16Zm3.857-9.809a.75.75 0 0 0-1.214-.882l-3.483 4.79-1.88-1.88a.75.75 0 1 0-1.06 1.061l2.5 2.5a.75.75 0 0 0 1.137-.089l4-5.5Z" clip-rule="evenodd" />
                                                                                        </svg>
                                                                                        <span>"Bajo"</span>
                                                                                    </span>
                                                                                }.into_view(),
                                                                            }}
                                                                        </td>

                                                                        // District
                                                                        <td class="py-3.5 px-3">
                                                                            <span class="text-xs font-semibold text-slate-700">
                                                                                {a.district.display_name()}
                                                                            </span>
                                                                        </td>

                                                                        // SLA Badge
                                                                        <td class="py-3.5 px-3">
                                                                            <span class={format!("px-2 py-0.5 rounded-full text-[10px] border inline-block whitespace-nowrap {}", sla_class)}>
                                                                                {sla_text}
                                                                            </span>
                                                                        </td>

                                                                        // Status
                                                                        <td class="py-3.5 px-3">
                                                                            <span class={format!("px-2 py-0.5 rounded text-[10px] font-bold border {}", match a.status.as_str() {
                                                                                "solved" => "bg-emerald-50 text-emerald-800 border-emerald-200",
                                                                                "dismissed" => "bg-slate-100 text-slate-600 border-slate-200",
                                                                                "verified" => "bg-purple-50 text-purple-800 border-purple-200 font-bold",
                                                                                _ => "bg-amber-50 text-amber-800 border-amber-200",
                                                                            })}>
                                                                                {match a.status.as_str() {
                                                                                    "solved" => "Resuelto",
                                                                                    "dismissed" => "Descartado",
                                                                                    "verified" => "En Inspección",
                                                                                    _ => "Pendiente",
                                                                                }}
                                                                            </span>
                                                                        </td>

                                                                        // Actions
                                                                        <td class="py-3.5 px-4 text-right">
                                                                            <div class="flex items-center justify-end space-x-1.5 flex-wrap gap-y-1">
                                                                                // 0. Ver en Mapa Interactivo
                                                                                <button
                                                                                    class="px-2.5 py-1 bg-emerald-50 hover:bg-emerald-100 text-emerald-800 border border-emerald-200 rounded-lg font-semibold text-[11px] transition flex items-center space-x-1 cursor-pointer shadow-2xs"
                                                                                    title="Ubicar e inspeccionar esta denuncia en el mapa general"
                                                                                    on:click={
                                                                                        let a_id = a.id.clone();
                                                                                        let a_lat = a.latitude;
                                                                                        let a_lng = a.longitude;
                                                                                        move |_| {
                                                                                            selected_alert_id.set(Some(a_id.clone()));
                                                                                            show_dashboard.set(false);
                                                                                            let _ = js_sys::eval(&format!(
                                                                                                "if (window.ecoMap && window.ecoMap.flyTo) window.ecoMap.flyTo({}, {}, 16.5);",
                                                                                                a_lat, a_lng
                                                                                            ));
                                                                                        }
                                                                                    }
                                                                                >
                                                                                    <i class="fa-solid fa-map-location-dot text-xs text-emerald-600"></i>
                                                                                    <span>"Ver en Mapa"</span>
                                                                                </button>

                                                                                // 1. Acta Oficial
                                                                                <button
                                                                                    class="px-2 py-1 bg-sky-50 text-sky-800 hover:bg-sky-100 border border-sky-200 rounded-lg font-semibold text-[11px] transition flex items-center space-x-1 cursor-pointer"
                                                                                    title="Ver Acta Técnica de Fiscalización"
                                                                                    on:click={
                                                                                        let a_for_acta = a.clone();
                                                                                        move |_| inspection_target.set(Some(a_for_acta.clone()))
                                                                                    }
                                                                                >
                                                                                    <i class="fa-solid fa-file-contract text-xs"></i>
                                                                                    <span>"Acta"</span>
                                                                                </button>

                                                                                // 1.1 Descargar PDF Directo
                                                                                <button
                                                                                    class="px-2 py-1 bg-emerald-50 text-emerald-800 hover:bg-emerald-100 border border-emerald-200 rounded-lg font-semibold text-[11px] transition flex items-center space-x-1 cursor-pointer"
                                                                                    title="Descargar directamente el Acta Pericial en formato PDF con membrete y QR oficial"
                                                                                    on:click={
                                                                                        let a_for_pdf = a.clone();
                                                                                        move |_| {
                                                                                            if let Ok(json_str) = serde_json::to_string(&a_for_pdf) {
                                                                                                let script = format!("window.ecoPdf.generateInspectionActa({});", json_str);
                                                                                                let _ = js_sys::eval(&script);
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                >
                                                                                    <i class="fa-solid fa-file-pdf text-xs"></i>
                                                                                    <span>"PDF"</span>
                                                                                </button>

                                                                                // 2. Historial / Auditoría
                                                                                <button
                                                                                    class="px-2 py-1 bg-purple-50 text-purple-800 hover:bg-purple-100 border border-purple-200 rounded-lg font-semibold text-[11px] transition flex items-center space-x-1 cursor-pointer"
                                                                                    title="Ver bitácora y trazabilidad de cambios"
                                                                                    on:click={
                                                                                        let a_for_hist = a.clone();
                                                                                        move |_| history_target.set(Some(a_for_hist.clone()))
                                                                                    }
                                                                                >
                                                                                    <span>"📜"</span>
                                                                                    <span>"Historial"</span>
                                                                                </button>

                                                                                // 3. Status-specific actions
                                                                                {if a.status == "pending" {
                                                                                    view! {
                                                                                        <button
                                                                                            class="px-2 py-1 bg-amber-50 hover:bg-amber-100 text-amber-800 border border-amber-200 rounded-lg font-semibold text-[11px] transition flex items-center space-x-1"
                                                                                            title="Asignar cuadrilla municipal e iniciar inspección"
                                                                                            on:click={
                                                                                                let a_for_squad = a.clone();
                                                                                                move |_| {
                                                                                                    action_error.set(None);
                                                                                                    assign_squad_target.set(Some(a_for_squad.clone()));
                                                                                                }
                                                                                            }
                                                                                        >
                                                                                            <span>"👷"</span>
                                                                                            <span>"Asignar"</span>
                                                                                        </button>
                                                                                        <button
                                                                                            class="px-2.5 py-1 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg font-semibold text-[11px] shadow-2xs transition"
                                                                                            title="Resolver incidencia con informe y foto"
                                                                                            on:click=move |_| {
                                                                                                resolve_comment.set("".to_string());
                                                                                                resolve_photo_url.set(None);
                                                                                                action_error.set(None);
                                                                                                resolve_target.set(Some(alert_for_resolve.clone()));
                                                                                            }
                                                                                        >
                                                                                            "Resolver"
                                                                                        </button>
                                                                                        <button
                                                                                            class="px-2 py-1 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg font-semibold text-[11px] transition"
                                                                                            title="Descartar reporte"
                                                                                            on:click=move |_| {
                                                                                                dismiss_reason.set("".to_string());
                                                                                                action_error.set(None);
                                                                                                dismiss_target.set(Some(alert_for_dismiss.clone()));
                                                                                            }
                                                                                        >
                                                                                            "Descartar"
                                                                                        </button>
                                                                                        <button
                                                                                            class="px-2 py-1 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg font-semibold text-[11px] transition"
                                                                                            title="Transferir a otro distrito"
                                                                                            on:click=move |_| {
                                                                                                transfer_district.set(alert_for_transfer.district);
                                                                                                action_error.set(None);
                                                                                                transfer_target.set(Some(alert_for_transfer.clone()));
                                                                                            }
                                                                                        >
                                                                                            "Transferir"
                                                                                        </button>
                                                                                    }.into_view()
                                                                                } else if a.status == "verified" {
                                                                                    view! {
                                                                                        <button
                                                                                            class="px-2 py-1 bg-amber-50 hover:bg-amber-100 text-amber-800 border border-amber-200 rounded-lg font-semibold text-[11px] transition flex items-center space-x-1"
                                                                                            title="Reasignar cuadrilla o actualizar orden"
                                                                                            on:click={
                                                                                                let a_for_squad = a.clone();
                                                                                                move |_| {
                                                                                                    action_error.set(None);
                                                                                                    assign_squad_target.set(Some(a_for_squad.clone()));
                                                                                                }
                                                                                            }
                                                                                        >
                                                                                            <span>"👷"</span>
                                                                                            <span>"Reasignar"</span>
                                                                                        </button>
                                                                                        <button
                                                                                            class="px-2.5 py-1 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg font-semibold text-[11px] shadow-2xs transition"
                                                                                            title="Cerrar y resolver incidencia"
                                                                                            on:click=move |_| {
                                                                                                resolve_comment.set("".to_string());
                                                                                                resolve_photo_url.set(None);
                                                                                                action_error.set(None);
                                                                                                resolve_target.set(Some(alert_for_resolve.clone()));
                                                                                            }
                                                                                        >
                                                                                            "Resolver"
                                                                                        </button>
                                                                                    }.into_view()
                                                                                } else {
                                                                                    view! { <div></div> }.into_view()
                                                                                }}
                                                                            </div>
                                                                        </td>
                                                                    </tr>
                                                                }
                                                            }).collect_view()
                                                        }
                                                    }}
                                                </tbody>
                                            </table>
                                        </div>
                                    </div>
                                </div>
                            }.into_view()
                        } else {
                            // Tab 1: Puntos de Acopio & Contenedores
                            view! {
                                <div class="space-y-6 animate-in fade-in duration-150">
                                    // Waste KPIs
                                    <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
                                        <div class="p-3.5 rounded-2xl bg-white border border-slate-200 shadow-xs">
                                            <p class="text-xs font-medium text-slate-500">"Total Puntos de Acopio"</p>
                                            <h3 class="text-xl font-bold text-slate-900 font-mono mt-0.5">{total_waste_count}</h3>
                                            <p class="text-[10px] text-slate-400">"Contenedores registrados"</p>
                                        </div>
                                        <div class="p-3.5 rounded-2xl bg-white border border-red-200/80 shadow-xs">
                                            <p class="text-xs font-medium text-red-700">"🔴 Llenos (Críticos)"</p>
                                            <h3 class="text-xl font-bold text-red-900 font-mono mt-0.5">{full_waste_count}</h3>
                                            <p class="text-[10px] text-red-600/80">"Requieren recojo urgente"</p>
                                        </div>
                                        <div class="p-3.5 rounded-2xl bg-white border border-amber-200/80 shadow-xs">
                                            <p class="text-xs font-medium text-amber-700">"🟡 Nivel Medio"</p>
                                            <h3 class="text-xl font-bold text-amber-900 font-mono mt-0.5">{medium_waste_count}</h3>
                                            <p class="text-[10px] text-amber-600/80">"50% - 70% capacidad"</p>
                                        </div>
                                        <div class="p-3.5 rounded-2xl bg-white border border-emerald-200/80 shadow-xs">
                                            <p class="text-xs font-medium text-emerald-700">"🟢 Disponibles (Bajos)"</p>
                                            <h3 class="text-xl font-bold text-emerald-900 font-mono mt-0.5">{low_waste_count}</h3>
                                            <p class="text-[10px] text-emerald-600/80">"Capacidad óptima"</p>
                                        </div>
                                    </div>

                                    // Filter Bar & Action buttons
                                    <div class="p-4 bg-white rounded-2xl border border-slate-200 shadow-xs space-y-3">
                                        <div class="flex flex-col sm:flex-row gap-3 items-center justify-between">
                                            <div class="relative w-full sm:w-72">
                                                <svg class="w-4 h-4 text-slate-400 absolute left-3 top-1/2 transform -translate-y-1/2" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                    <circle cx="11" cy="11" r="8"></circle>
                                                    <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                                                </svg>
                                                <input
                                                    type="text"
                                                    placeholder="Buscar punto de acopio, contenedor o calle..."
                                                    class="w-full pl-9 pr-3 py-1.5 text-xs bg-slate-50 border border-slate-200 rounded-xl focus:bg-white focus:outline-hidden focus:ring-2 focus:ring-slate-900 transition"
                                                    prop:value=move || waste_search_query.get()
                                                    on:input=move |e| waste_search_query.set(event_target_value(&e))
                                                />
                                            </div>

                                            <div class="flex items-center space-x-2 w-full sm:w-auto justify-end">
                                                <button
                                                    class="px-3.5 py-2 text-xs font-semibold bg-teal-50 hover:bg-teal-100 text-teal-800 border border-teal-200 rounded-xl transition flex items-center space-x-1.5 cursor-pointer shadow-2xs"
                                                    title="Ver ruta del camión recolector y puntos de acopio en el mapa interactivo"
                                                    on:click=move |_| {
                                                        show_waste_layer.set(true);
                                                        show_dashboard.set(false);
                                                    }
                                                >
                                                    <i class="fa-solid fa-truck-moving text-teal-600 text-xs"></i>
                                                    <span>"Ver Ruta & Puntos en el Mapa"</span>
                                                </button>

                                                <button
                                                    class="px-3.5 py-2 text-xs font-semibold bg-emerald-600 hover:bg-emerald-700 text-white rounded-xl shadow-xs transition flex items-center space-x-1.5 cursor-pointer"
                                                    on:click=open_create_waste_point
                                                >
                                                    <svg class="w-4 h-4 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                        <line x1="12" y1="5" x2="12" y2="19"></line>
                                                        <line x1="5" y1="12" x2="19" y2="12"></line>
                                                    </svg>
                                                    <span>"+ Nuevo Punto"</span>
                                                </button>
                                            </div>
                                        </div>

                                        // Sub-filters for Fill Level and Waste Type
                                        <div class="flex flex-wrap items-center justify-between gap-2 pt-2 border-t border-slate-100 text-xs">
                                            <div class="flex items-center space-x-1.5 overflow-x-auto">
                                                <span class="text-[10px] font-bold text-slate-400 uppercase tracking-wider mr-1">"Nivel:"</span>
                                                <button
                                                    class=move || if filter_waste_fill.get() == "todos" { "px-2.5 py-1 rounded-lg bg-slate-900 text-white font-semibold" } else { "px-2.5 py-1 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium" }
                                                    on:click=move |_| filter_waste_fill.set("todos".to_string())
                                                >
                                                    "Todos"
                                                </button>
                                                <button
                                                    class=move || if filter_waste_fill.get() == "full" { "px-2.5 py-1 rounded-lg bg-red-600 text-white font-semibold" } else { "px-2.5 py-1 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium" }
                                                    on:click=move |_| filter_waste_fill.set("full".to_string())
                                                >
                                                    "🔴 Llenos"
                                                </button>
                                                <button
                                                    class=move || if filter_waste_fill.get() == "medium" { "px-2.5 py-1 rounded-lg bg-amber-600 text-white font-semibold" } else { "px-2.5 py-1 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium" }
                                                    on:click=move |_| filter_waste_fill.set("medium".to_string())
                                                >
                                                    "🟡 Medios"
                                                </button>
                                                <button
                                                    class=move || if filter_waste_fill.get() == "low" { "px-2.5 py-1 rounded-lg bg-emerald-600 text-white font-semibold" } else { "px-2.5 py-1 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium" }
                                                    on:click=move |_| filter_waste_fill.set("low".to_string())
                                                >
                                                    "🟢 Bajos"
                                                </button>
                                            </div>

                                            <div class="flex items-center space-x-1.5 overflow-x-auto">
                                                <span class="text-[10px] font-bold text-slate-400 uppercase tracking-wider mr-1">"Tipo:"</span>
                                                <button
                                                    class=move || if filter_waste_type.get() == "todos" { "px-2 py-0.5 rounded-lg bg-slate-900 text-white font-semibold" } else { "px-2 py-0.5 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium" }
                                                    on:click=move |_| filter_waste_type.set("todos".to_string())
                                                >
                                                    "Todos"
                                                </button>
                                                <button
                                                    class=move || if filter_waste_type.get() == "recyclable" { "px-2 py-0.5 rounded-lg bg-emerald-700 text-white font-semibold" } else { "px-2 py-0.5 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium" }
                                                    on:click=move |_| filter_waste_type.set("recyclable".to_string())
                                                >
                                                    "Reciclables"
                                                </button>
                                                <button
                                                    class=move || if filter_waste_type.get() == "organic" { "px-2 py-0.5 rounded-lg bg-amber-700 text-white font-semibold" } else { "px-2 py-0.5 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium" }
                                                    on:click=move |_| filter_waste_type.set("organic".to_string())
                                                >
                                                    "Orgánicos"
                                                </button>
                                                <button
                                                    class=move || if filter_waste_type.get() == "general" { "px-2 py-0.5 rounded-lg bg-slate-700 text-white font-semibold" } else { "px-2 py-0.5 rounded-lg bg-slate-100 text-slate-600 hover:bg-slate-200 font-medium" }
                                                    on:click=move |_| filter_waste_type.set("general".to_string())
                                                >
                                                    "Generales"
                                                </button>
                                            </div>
                                        </div>
                                    </div>

                                    // Containers Grid
                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                        {move || {
                                            let list = displayed_waste_points.get();
                                            if list.is_empty() {
                                                view! {
                                                    <div class="col-span-full p-12 bg-white rounded-2xl border border-slate-200 text-center space-y-2">
                                                        <div class="text-3xl">"🗑️"</div>
                                                        <h4 class="text-sm font-bold text-slate-800">"No se encontraron puntos de acopio"</h4>
                                                        <p class="text-xs text-slate-500">"Prueba cambiando los filtros de búsqueda o registra un nuevo punto."</p>
                                                    </div>
                                                }.into_view()
                                            } else {
                                                list.into_iter().map(|wp| {
                                                    let pid = wp.id_string();
                                                    let pid_delete = pid.clone();
                                                    let pid_low = pid.clone();
                                                    let pid_med = pid.clone();
                                                    let pid_full = pid.clone();
                                                    let wp_edit = wp.clone();
                                                    let wp_map = wp.clone();

                                                    view! {
                                                        <div class="p-4 bg-white rounded-2xl border border-slate-200 shadow-xs space-y-3 hover:border-slate-300 transition">
                                                            <div class="flex items-start justify-between gap-3">
                                                                <div class="min-w-0">
                                                                    <div class="flex items-center space-x-2">
                                                                        <span class={format!("px-2 py-0.5 rounded text-[10px] font-bold border {}", wp.waste_type.color_class())}>
                                                                            {wp.waste_type.display_name()}
                                                                        </span>
                                                                        <span class={format!("px-2 py-0.5 rounded text-[10px] font-bold border {}", wp.fill_level.badge_class())}>
                                                                            {wp.fill_level.display_name()}
                                                                        </span>
                                                                    </div>
                                                                    <h4 class="text-xs font-bold text-slate-900 mt-1 leading-snug truncate">
                                                                        {wp.name.clone()}
                                                                    </h4>
                                                                    <button
                                                                        type="button"
                                                                        class="text-[11px] text-slate-500 hover:text-emerald-700 flex items-center space-x-1 truncate mt-0.5 text-left group cursor-pointer transition"
                                                                        title="Ver este punto en el mapa interactivo"
                                                                        on:click={
                                                                            let m_wp = wp_map.clone();
                                                                            move |_| {
                                                                                show_waste_layer.set(true);
                                                                                selected_waste_point_id.set(Some(m_wp.id_string()));
                                                                                show_dashboard.set(false);
                                                                                let _ = js_sys::eval(&format!(
                                                                                    "if (window.ecoMap && window.ecoMap.flyTo) window.ecoMap.flyTo({}, {}, 16.5);",
                                                                                    m_wp.latitude, m_wp.longitude
                                                                                ));
                                                                            }
                                                                        }
                                                                    >
                                                                        <i class="fa-solid fa-location-dot text-slate-400 group-hover:text-emerald-600 text-[10px]"></i>
                                                                        <span class="truncate">{wp.address.clone()}</span>
                                                                    </button>
                                                                </div>

                                                                <div class="flex items-center space-x-1 shrink-0">
                                                                    <button
                                                                        class="p-1.5 text-slate-400 hover:text-emerald-700 rounded-lg hover:bg-emerald-50 transition cursor-pointer"
                                                                        title="Ver en el mapa general"
                                                                        on:click={
                                                                            let m_wp = wp_map.clone();
                                                                            move |_| {
                                                                                show_waste_layer.set(true);
                                                                                selected_waste_point_id.set(Some(m_wp.id_string()));
                                                                                show_dashboard.set(false);
                                                                                let _ = js_sys::eval(&format!(
                                                                                    "if (window.ecoMap && window.ecoMap.flyTo) window.ecoMap.flyTo({}, {}, 16.5);",
                                                                                    m_wp.latitude, m_wp.longitude
                                                                                ));
                                                                            }
                                                                        }
                                                                    >
                                                                        <i class="fa-solid fa-map-location-dot text-sm text-emerald-600"></i>
                                                                    </button>
                                                                    <button
                                                                        class="p-1.5 text-slate-400 hover:text-sky-600 rounded-lg hover:bg-sky-50 transition cursor-pointer"
                                                                        title="Editar este punto de acopio"
                                                                        on:click={
                                                                            let wp_clone = wp_edit.clone();
                                                                            move |_| open_edit_waste_point(wp_clone.clone())
                                                                        }
                                                                    >
                                                                        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                                            <path d="M12 20h9"/>
                                                                            <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/>
                                                                        </svg>
                                                                    </button>
                                                                    <button
                                                                        class="p-1.5 text-slate-400 hover:text-red-600 rounded-lg hover:bg-red-50 transition cursor-pointer"
                                                                        title="Eliminar este punto de acopio"
                                                                        on:click=move |_| delete_waste_point(pid_delete.clone())
                                                                    >
                                                                        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                                            <polyline points="3 6 5 6 21 6"></polyline>
                                                                            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                                                                        </svg>
                                                                    </button>
                                                                </div>
                                                            </div>

                                                            <div class="p-2.5 rounded-xl bg-slate-50 border border-slate-100 text-[11px] text-slate-600">
                                                                <p class="font-bold text-slate-800 text-[10px] uppercase tracking-wider">"Horario del Camión:"</p>
                                                                <p class="mt-0.5">{wp.truck_schedule.clone()}</p>
                                                            </div>

                                                            // Quick Fill Level Adjustment Buttons
                                                            <div class="space-y-1.5 pt-1">
                                                                <span class="text-[10px] font-bold text-slate-400 uppercase tracking-wider block">
                                                                    "Actualizar Nivel de Llenado:"
                                                                </span>
                                                                <div class="flex space-x-1.5">
                                                                    <button
                                                                        class=move || {
                                                                            if wp.fill_level == FillLevel::Low {
                                                                                "flex-1 py-1 rounded-lg text-xs font-bold bg-emerald-600 text-white shadow-2xs"
                                                                            } else {
                                                                                "flex-1 py-1 rounded-lg text-xs font-medium bg-slate-100 hover:bg-slate-200 text-slate-700 transition"
                                                                            }
                                                                        }
                                                                        on:click=move |_| update_waste_fill_level(pid_low.clone(), FillLevel::Low)
                                                                    >
                                                                        "Bajo"
                                                                    </button>
                                                                    <button
                                                                        class=move || {
                                                                            if wp.fill_level == FillLevel::Medium {
                                                                                "flex-1 py-1 rounded-lg text-xs font-bold bg-amber-600 text-white shadow-2xs"
                                                                            } else {
                                                                                "flex-1 py-1 rounded-lg text-xs font-medium bg-slate-100 hover:bg-slate-200 text-slate-700 transition"
                                                                            }
                                                                        }
                                                                        on:click=move |_| update_waste_fill_level(pid_med.clone(), FillLevel::Medium)
                                                                    >
                                                                        "Medio"
                                                                    </button>
                                                                    <button
                                                                        class=move || {
                                                                            if wp.fill_level == FillLevel::Full {
                                                                                "flex-1 py-1 rounded-lg text-xs font-bold bg-red-600 text-white shadow-2xs animate-pulse"
                                                                            } else {
                                                                                "flex-1 py-1 rounded-lg text-xs font-medium bg-slate-100 hover:bg-slate-200 text-slate-700 transition"
                                                                            }
                                                                        }
                                                                        on:click=move |_| update_waste_fill_level(pid_full.clone(), FillLevel::Full)
                                                                    >
                                                                        "Lleno"
                                                                    </button>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    }
                                                }).collect_view()
                                            }
                                        }}
                                    </div>
                                </div>
                            }.into_view()
                        }}
                    </main>

                    // MODAL 1: RESOLVER ALERTA
                    {move || resolve_target.get().map(|target| {
                        view! {
                            <div class="fixed inset-0 z-50 bg-slate-900/40 backdrop-blur-sm flex items-center justify-center p-4">
                                <div class="bg-white w-full max-w-lg rounded-2xl border border-slate-200 p-6 shadow-xl space-y-4">
                                    <div class="flex items-center justify-between border-b border-slate-100 pb-3">
                                        <div class="flex items-center space-x-2">
                                            <div class="w-8 h-8 rounded-lg bg-emerald-100 text-emerald-700 flex items-center justify-center">
                                                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                                                    <polyline points="20 6 9 17 4 12"></polyline>
                                                </svg>
                                            </div>
                                            <h3 class="text-sm font-bold text-slate-900">"Resolver Denuncia Ambiental"</h3>
                                        </div>
                                        <button class="text-slate-400 hover:text-slate-600 p-1" on:click=move |_| resolve_target.set(None)>
                                            "✕"
                                        </button>
                                    </div>

                                    <div class="p-3 bg-slate-50 rounded-xl border border-slate-100 text-xs">
                                        <p class="font-bold text-slate-800">{target.title.clone()}</p>
                                        <p class="text-slate-500 text-[11px] truncate mt-0.5">{target.address.clone()}</p>
                                    </div>

                                    // Error Banner
                                    {move || action_error.get().map(|e| view! {
                                        <div class="p-2.5 bg-red-50 text-red-700 text-xs rounded-xl border border-red-200">
                                            {e}
                                        </div>
                                    })}

                                    // Comment
                                    <div class="space-y-1">
                                        <label class="text-[11px] font-bold text-slate-700 uppercase tracking-wider block">
                                            "Informe Oficial de Trabajo Realizado *"
                                        </label>
                                        <textarea
                                            rows="3"
                                            placeholder="Detalla las cuadrillas asignadas, maquinaria utilizada, toneladas de residuos retiradas o neutralización realizada..."
                                            class="w-full p-2.5 text-xs bg-slate-50 border border-slate-200 rounded-xl focus:bg-white focus:outline-hidden focus:ring-2 focus:ring-slate-900 transition"
                                            prop:value=move || resolve_comment.get()
                                            on:input=move |e| resolve_comment.set(event_target_value(&e))
                                        ></textarea>
                                    </div>

                                    // Upload Resolution Evidence Photo
                                    <div class="space-y-1.5">
                                        <label class="text-[11px] font-bold text-slate-700 uppercase tracking-wider block">
                                            "Fotografía de Evidencia (DESPUÉS)"
                                        </label>
                                        {move || if let Some(url) = resolve_photo_url.get() {
                                            view! {
                                                <div class="relative w-full h-36 rounded-xl overflow-hidden bg-slate-100 border border-slate-200">
                                                    <img src=url.clone() class="w-full h-full object-cover" alt="Evidencia de resolución" />
                                                    <button
                                                        class="absolute top-2 right-2 p-1 rounded-full bg-red-600 text-white shadow-xs"
                                                        on:click=move |_| resolve_photo_url.set(None)
                                                    >
                                                        "✕"
                                                    </button>
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <button
                                                    type="button"
                                                    class="w-full py-3 border-2 border-dashed border-slate-200 hover:border-slate-400 rounded-xl text-xs font-semibold text-slate-600 hover:text-slate-800 transition flex items-center justify-center space-x-2"
                                                    disabled=move || is_uploading_resolve_photo.get()
                                                    on:click=upload_resolve_photo
                                                >
                                                    {move || if is_uploading_resolve_photo.get() {
                                                        view! {
                                                            <span class="inline-block w-4 h-4 border-2 border-slate-400 border-t-slate-800 rounded-full animate-spin"></span>
                                                            <span>"Subiendo evidencia WebP..."</span>
                                                        }.into_view()
                                                    } else {
                                                        view! {
                                                            <svg class="w-4 h-4 text-slate-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                                <rect width="18" height="18" x="3" y="3" rx="2" ry="2"/>
                                                                <circle cx="9" cy="9" r="2"/>
                                                                <path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/>
                                                            </svg>
                                                            <span>"Adjuntar Foto de Trabajos Realizados"</span>
                                                        }.into_view()
                                                    }}
                                                </button>
                                            }.into_view()
                                        }}
                                    </div>

                                    <div class="flex justify-end space-x-2 pt-2 border-t border-slate-100">
                                        <button class="px-3 py-1.5 text-xs text-slate-600 font-semibold" on:click=move |_| resolve_target.set(None)>
                                            "Cancelar"
                                        </button>
                                        <button
                                            class="px-4 py-2 text-xs font-semibold bg-emerald-600 hover:bg-emerald-700 text-white rounded-xl shadow-xs transition"
                                            disabled=move || is_submitting_action.get()
                                            on:click=submit_resolve
                                        >
                                            {move || if is_submitting_action.get() { "Guardando..." } else { "Confirmar Resolución" }}
                                        </button>
                                    </div>
                                </div>
                            </div>
                        }
                    })}

                    // MODAL 2: DESCARTAR ALERTA
                    {move || dismiss_target.get().map(|target| {
                        view! {
                            <div class="fixed inset-0 z-50 bg-slate-900/40 backdrop-blur-sm flex items-center justify-center p-4">
                                <div class="bg-white w-full max-w-md rounded-2xl border border-slate-200 p-6 shadow-xl space-y-4">
                                    <div class="flex items-center justify-between border-b border-slate-100 pb-3">
                                        <div class="flex items-center space-x-2">
                                            <div class="w-8 h-8 rounded-lg bg-red-100 text-red-700 flex items-center justify-center">
                                                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                    <circle cx="12" cy="12" r="10"></circle>
                                                    <line x1="15" y1="9" x2="9" y2="15"></line>
                                                    <line x1="9" y1="9" x2="15" y2="15"></line>
                                                </svg>
                                            </div>
                                            <h3 class="text-sm font-bold text-slate-900">"Descartar Alerta"</h3>
                                        </div>
                                        <button class="text-slate-400 hover:text-slate-600 p-1" on:click=move |_| dismiss_target.set(None)>
                                            "✕"
                                        </button>
                                    </div>

                                    <div class="p-3 bg-slate-50 rounded-xl border border-slate-100 text-xs">
                                        <p class="font-bold text-slate-800">{target.title.clone()}</p>
                                        <p class="text-slate-500 text-[11px] truncate mt-0.5">{target.address.clone()}</p>
                                    </div>

                                    <p class="text-xs text-slate-600">
                                        "Ingresa la justificación técnica por la cual esta denuncia no procede (ej. fuera de competencia o reporte duplicado):"
                                    </p>

                                    {move || action_error.get().map(|e| view! {
                                        <div class="p-2.5 bg-red-50 text-red-700 text-xs rounded-xl border border-red-200">
                                            {e}
                                        </div>
                                    })}

                                    <textarea
                                        rows="3"
                                        placeholder="Motivo de desestimación..."
                                        class="w-full p-2.5 text-xs bg-slate-50 border border-slate-200 rounded-xl focus:bg-white focus:outline-hidden focus:ring-2 focus:ring-slate-900 transition"
                                        prop:value=move || dismiss_reason.get()
                                        on:input=move |e| dismiss_reason.set(event_target_value(&e))
                                    ></textarea>

                                    <div class="flex justify-end space-x-2 pt-2 border-t border-slate-100">
                                        <button class="px-3 py-1.5 text-xs text-slate-600 font-semibold" on:click=move |_| dismiss_target.set(None)>
                                            "Cancelar"
                                        </button>
                                        <button
                                            class="px-4 py-2 text-xs font-semibold bg-red-600 hover:bg-red-700 text-white rounded-xl shadow-xs transition"
                                            disabled=move || is_submitting_action.get()
                                            on:click=submit_dismiss
                                        >
                                            {move || if is_submitting_action.get() { "Guardando..." } else { "Descartar Incidente" }}
                                        </button>
                                    </div>
                                </div>
                            </div>
                        }
                    })}

                    // MODAL 3: TRANSFERIR JURISDICCIÓN
                    {move || transfer_target.get().map(|target| {
                        view! {
                            <div class="fixed inset-0 z-50 bg-slate-900/40 backdrop-blur-sm flex items-center justify-center p-4">
                                <div class="bg-white w-full max-w-md rounded-2xl border border-slate-200 p-6 shadow-xl space-y-4">
                                    <div class="flex items-center justify-between border-b border-slate-100 pb-3">
                                        <div class="flex items-center space-x-2">
                                            <div class="w-8 h-8 rounded-lg bg-sky-100 text-sky-700 flex items-center justify-center">
                                                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                    <polyline points="17 1 21 5 17 9"></polyline>
                                                    <path d="M3 11V9a4 4 0 0 1 4-4h14"></path>
                                                    <polyline points="7 23 3 19 7 15"></polyline>
                                                    <path d="M21 13v2a4 4 0 0 1-4 4H3"></path>
                                                </svg>
                                            </div>
                                            <h3 class="text-sm font-bold text-slate-900">"Transferir Jurisdicción"</h3>
                                        </div>
                                        <button class="text-slate-400 hover:text-slate-600 p-1" on:click=move |_| transfer_target.set(None)>
                                            "✕"
                                        </button>
                                    </div>

                                    <div class="p-3 bg-slate-50 rounded-xl border border-slate-100 text-xs">
                                        <p class="font-bold text-slate-800">{target.title.clone()}</p>
                                        <p class="text-slate-500 text-[11px] mt-0.5">"Distrito actual: " <span class="font-bold">{target.district.display_name()}</span></p>
                                    </div>

                                    {move || action_error.get().map(|e| view! {
                                        <div class="p-2.5 bg-red-50 text-red-700 text-xs rounded-xl border border-red-200">
                                            {e}
                                        </div>
                                    })}

                                    <div class="space-y-1.5">
                                        <label class="text-[11px] font-bold text-slate-700 uppercase tracking-wider block">
                                            "Nuevo Distrito Competente:"
                                        </label>
                                        <div class="space-y-1.5">
                                            {EcoDistrict::all().iter().map(|&d| {
                                                let is_sel = move || transfer_district.get() == d;
                                                view! {
                                                    <div
                                                        class=move || {
                                                            if is_sel() {
                                                                "p-3 rounded-xl border-2 border-slate-900 bg-slate-50 text-xs font-bold text-slate-900 cursor-pointer flex items-center justify-between"
                                                            } else {
                                                                "p-3 rounded-xl border border-slate-200 hover:bg-slate-50 text-xs font-medium text-slate-700 cursor-pointer flex items-center justify-between"
                                                            }
                                                        }
                                                        on:click=move |_| transfer_district.set(d)
                                                    >
                                                        <span>{d.display_name()}</span>
                                                        {move || if is_sel() { "✓" } else { "" }}
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    </div>

                                    <div class="flex justify-end space-x-2 pt-2 border-t border-slate-100">
                                        <button class="px-3 py-1.5 text-xs text-slate-600 font-semibold" on:click=move |_| transfer_target.set(None)>
                                            "Cancelar"
                                        </button>
                                        <button
                                            class="px-4 py-2 text-xs font-semibold bg-slate-900 hover:bg-slate-800 text-white rounded-xl shadow-xs transition"
                                            disabled=move || is_submitting_action.get()
                                            on:click=submit_transfer
                                        >
                                            {move || if is_submitting_action.get() { "Transfiriendo..." } else { "Confirmar Transferencia" }}
                                        </button>
                                    </div>
                                </div>
                            </div>
                        }
                    })}

                    // MODAL 4: CREAR O EDITAR PUNTO DE ACOPIO
                    {move || show_create_waste_modal.get().then(|| {
                        view! {
                            <div class="fixed inset-0 z-50 bg-slate-900/40 backdrop-blur-sm flex items-center justify-center p-4">
                                <div class="bg-white w-full max-w-lg rounded-2xl border border-slate-200 p-6 shadow-xl space-y-4 max-h-[90vh] overflow-y-auto">
                                    <div class="flex items-center justify-between border-b border-slate-100 pb-3">
                                        <h3 class="text-sm font-bold text-slate-900">
                                            {move || if editing_waste_target.get().is_some() { "Editar Punto de Acopio" } else { "Registrar Nuevo Punto de Acopio" }}
                                        </h3>
                                        <button class="text-slate-400 hover:text-slate-600 p-1" on:click=move |_| show_create_waste_modal.set(false)>
                                            "✕"
                                        </button>
                                    </div>

                                    {move || action_error.get().map(|e| view! {
                                        <div class="p-2.5 bg-red-50 text-red-700 text-xs rounded-xl border border-red-200">
                                            {e}
                                        </div>
                                    })}

                                    <div class="space-y-3 text-xs">
                                        <div>
                                            <label class="font-bold text-slate-700 block mb-1">"Nombre del Punto *"</label>
                                            <input
                                                type="text"
                                                placeholder="Ej. Estación Ecológica - Plaza Simón Bolívar"
                                                class="w-full p-2 bg-slate-50 border border-slate-200 rounded-xl focus:bg-white"
                                                prop:value=move || new_waste_name.get()
                                                on:input=move |e| new_waste_name.set(event_target_value(&e))
                                            />
                                        </div>

                                        // Dirección con autocompletado y sugerencia mágica
                                        <div>
                                            <div class="flex items-center justify-between mb-1">
                                                <label class="font-bold text-slate-700 block">"Dirección / Referencia *"</label>
                                                <button
                                                    type="button"
                                                    class="text-[10.5px] text-emerald-700 hover:text-emerald-800 font-semibold flex items-center space-x-1 cursor-pointer transition"
                                                    disabled=move || is_reverse_geocoding_waste.get()
                                                    on:click=suggest_waste_address
                                                    title="Obtener el nombre de la calle automáticamente según el punto fijado en el mapa"
                                                >
                                                    <i class="fa-solid fa-wand-magic-sparkles text-[10px]"></i>
                                                    <span>{move || if is_reverse_geocoding_waste.get() { "Consultando..." } else { "Sugerir según mapa" }}</span>
                                                </button>
                                            </div>
                                            <input
                                                type="text"
                                                placeholder="Ej. Av. Los Próceres cuadra 2, Yanacancha"
                                                class="w-full p-2 bg-slate-50 border border-slate-200 rounded-xl focus:bg-white text-xs"
                                                prop:value=move || new_waste_addr.get()
                                                on:input=move |e| new_waste_addr.set(event_target_value(&e))
                                            />
                                        </div>

                                        // Selector de Ubicación Interactivo con Mapa Leaflet
                                        <div class="space-y-2 p-3 bg-slate-50 rounded-2xl border border-slate-200">
                                            <div class="flex items-center justify-between">
                                                <div>
                                                    <label class="font-bold text-slate-800 block text-xs">
                                                        "Ubicación en el Mapa Interactivo *"
                                                    </label>
                                                    <p class="text-[10.5px] text-slate-500">
                                                        "Haz clic en el mapa o arrastra el marcador para fijar la ubicación del contenedor."
                                                    </p>
                                                </div>

                                                <button
                                                    type="button"
                                                    class="px-2.5 py-1 bg-slate-900 hover:bg-slate-800 text-white rounded-lg font-semibold text-[11px] transition flex items-center space-x-1 cursor-pointer shadow-2xs shrink-0"
                                                    disabled=move || is_locating_waste_gps.get()
                                                    on:click=locate_waste_gps
                                                    title="Ubicar el punto usando tu señal GPS del dispositivo"
                                                >
                                                    <i class="fa-solid fa-location-crosshairs text-emerald-400 text-xs"></i>
                                                    <span>
                                                        {move || if is_locating_waste_gps.get() { "Detectando..." } else { "Usar Mi GPS" }}
                                                    </span>
                                                </button>
                                            </div>

                                            // Contenedor del Mapa Leaflet Interactivo
                                            <div
                                                id="waste-picker-map"
                                                class="w-full h-48 rounded-xl border border-slate-300 shadow-inner z-0 relative overflow-hidden bg-slate-200"
                                            ></div>

                                            // Zonas Rápidas de Cerro de Pasco
                                            <div class="space-y-1 pt-1">
                                                <span class="text-[10px] font-bold text-slate-400 uppercase tracking-wider block">
                                                    "Zonas Estratégicas de Cerro de Pasco (Clic para posicionar):"
                                                </span>
                                                <div class="flex flex-wrap gap-1.5">
                                                    <button
                                                        type="button"
                                                        class="px-2 py-1 rounded-lg bg-white hover:bg-slate-100 border border-slate-200 text-slate-700 text-[10.5px] font-medium transition cursor-pointer shadow-2xs"
                                                        on:click=move |_| set_waste_preset("Plaza Daniel Alcides Carrión, Chaupimarca", -10.6781, -76.2545)
                                                    >
                                                        "🏛️ Chaupimarca"
                                                    </button>
                                                    <button
                                                        type="button"
                                                        class="px-2 py-1 rounded-lg bg-white hover:bg-slate-100 border border-slate-200 text-slate-700 text-[10.5px] font-medium transition cursor-pointer shadow-2xs"
                                                        on:click=move |_| set_waste_preset("Av. Los Próceres (San Juan Pampa), Yanacancha", -10.6550, -76.2410)
                                                    >
                                                        "🏢 Yanacancha"
                                                    </button>
                                                    <button
                                                        type="button"
                                                        class="px-2 py-1 rounded-lg bg-white hover:bg-slate-100 border border-slate-200 text-slate-700 text-[10.5px] font-medium transition cursor-pointer shadow-2xs"
                                                        on:click=move |_| set_waste_preset("Laguna Patarcocha Ribera Este, Chaupimarca", -10.6655, -76.2525)
                                                    >
                                                        "🌊 Patarcocha"
                                                    </button>
                                                    <button
                                                        type="button"
                                                        class="px-2 py-1 rounded-lg bg-white hover:bg-slate-100 border border-slate-200 text-slate-700 text-[10.5px] font-medium transition cursor-pointer shadow-2xs"
                                                        on:click=move |_| set_waste_preset("Plaza Principal de Paragsha", -10.6720, -76.2730)
                                                    >
                                                        "🏭 Paragsha"
                                                    </button>
                                                    <button
                                                        type="button"
                                                        class="px-2 py-1 rounded-lg bg-white hover:bg-slate-100 border border-slate-200 text-slate-700 text-[10.5px] font-medium transition cursor-pointer shadow-2xs"
                                                        on:click=move |_| set_waste_preset("Sector Bellavista s/n, Simón Bolívar", -10.6970, -76.2750)
                                                    >
                                                        "🚜 Simón Bolívar"
                                                    </button>
                                                    <button
                                                        type="button"
                                                        class="px-2 py-1 rounded-lg bg-white hover:bg-slate-100 border border-slate-200 text-slate-700 text-[10.5px] font-medium transition cursor-pointer shadow-2xs"
                                                        on:click=move |_| set_waste_preset("Ciudad Universitaria UNDAC, Yanacancha", -10.6690, -76.2520)
                                                    >
                                                        "🎓 UNDAC"
                                                    </button>
                                                </div>
                                            </div>

                                            // Coordenadas fijadas automáticamente con opción de ajuste numérico
                                            <div class="flex items-center justify-between pt-1 border-t border-slate-200/80 text-[11px]">
                                                <span class="font-mono text-slate-600 bg-white px-2 py-0.5 rounded-md border border-slate-200 text-[10px] shadow-2xs">
                                                    {move || format!("📍 Lat: {:.5}, Lng: {:.5}", new_waste_lat.get(), new_waste_lng.get())}
                                                </span>

                                                <button
                                                    type="button"
                                                    class="text-[10.5px] text-slate-500 hover:text-slate-800 underline cursor-pointer"
                                                    on:click=move |_| show_manual_coords_details.update(|v| *v = !*v)
                                                >
                                                    {move || if show_manual_coords_details.get() { "Ocultar coordenadas numéricas" } else { "Ajuste numérico manual (Opcional)" }}
                                                </button>
                                            </div>

                                            {move || show_manual_coords_details.get().then(|| {
                                                view! {
                                                    <div class="grid grid-cols-2 gap-2 pt-2 border-t border-slate-200">
                                                        <div>
                                                            <label class="text-[10px] font-bold text-slate-500 block mb-0.5">"Latitud GPS"</label>
                                                            <input
                                                                type="number"
                                                                step="0.0001"
                                                                class="w-full p-1.5 bg-white border border-slate-200 rounded-lg font-mono text-xs"
                                                                prop:value=move || new_waste_lat.get().to_string()
                                                                on:input=move |e| {
                                                                    if let Ok(v) = event_target_value(&e).parse::<f64>() {
                                                                        new_waste_lat.set(v);
                                                                        let _ = js_sys::eval(&format!(
                                                                            "if (window.ecoWastePicker) window.ecoWastePicker.setPosition({}, {});",
                                                                            v, new_waste_lng.get_untracked()
                                                                        ));
                                                                    }
                                                                }
                                                            />
                                                        </div>
                                                        <div>
                                                            <label class="text-[10px] font-bold text-slate-500 block mb-0.5">"Longitud GPS"</label>
                                                            <input
                                                                type="number"
                                                                step="0.0001"
                                                                class="w-full p-1.5 bg-white border border-slate-200 rounded-lg font-mono text-xs"
                                                                prop:value=move || new_waste_lng.get().to_string()
                                                                on:input=move |e| {
                                                                    if let Ok(v) = event_target_value(&e).parse::<f64>() {
                                                                        new_waste_lng.set(v);
                                                                        let _ = js_sys::eval(&format!(
                                                                            "if (window.ecoWastePicker) window.ecoWastePicker.setPosition({}, {});",
                                                                            new_waste_lat.get_untracked(), v
                                                                        ));
                                                                    }
                                                                }
                                                            />
                                                        </div>
                                                    </div>
                                                }
                                            })}
                                        </div>

                                        <div class="grid grid-cols-2 gap-2">
                                            <div>
                                                <label class="font-bold text-slate-700 block mb-1">"Tipo de Residuo"</label>
                                                <select
                                                    class="w-full p-2 bg-slate-50 border border-slate-200 rounded-xl font-medium"
                                                    prop:value=move || new_waste_type.get().as_str()
                                                    on:change=move |e| {
                                                        match event_target_value(&e).as_str() {
                                                            "organic" => new_waste_type.set(WasteType::Organic),
                                                            "general" => new_waste_type.set(WasteType::General),
                                                            _ => new_waste_type.set(WasteType::Recyclable),
                                                        }
                                                    }
                                                >
                                                    <option value="recyclable">"Reciclables (Verde)"</option>
                                                    <option value="organic">"Orgánicos (Marrón)"</option>
                                                    <option value="general">"Generales (Gris)"</option>
                                                </select>
                                            </div>

                                            <div>
                                                <label class="font-bold text-slate-700 block mb-1">"Nivel de Llenado"</label>
                                                <select
                                                    class="w-full p-2 bg-slate-50 border border-slate-200 rounded-xl font-medium"
                                                    prop:value=move || new_waste_fill.get().as_str()
                                                    on:change=move |e| {
                                                        match event_target_value(&e).as_str() {
                                                            "medium" => new_waste_fill.set(FillLevel::Medium),
                                                            "full" => new_waste_fill.set(FillLevel::Full),
                                                            _ => new_waste_fill.set(FillLevel::Low),
                                                        }
                                                    }
                                                >
                                                    <option value="low">"Bajo (<30%)"</option>
                                                    <option value="medium">"Medio (50-70%)"</option>
                                                    <option value="full">"Lleno (Capacidad Máxima)"</option>
                                                </select>
                                            </div>
                                        </div>

                                        <div>
                                            <label class="font-bold text-slate-700 block mb-1">"Horario del Camión Recolector"</label>
                                            <input
                                                type="text"
                                                placeholder="Ej. Lunes, Miércoles y Viernes de 07:00 a 09:00"
                                                class="w-full p-2 bg-slate-50 border border-slate-200 rounded-xl focus:bg-white"
                                                prop:value=move || new_waste_schedule.get()
                                                on:input=move |e| new_waste_schedule.set(event_target_value(&e))
                                            />
                                        </div>

                                        <div>
                                            <label class="font-bold text-slate-700 block mb-1">"Recomendaciones Ciudadanas"</label>
                                            <textarea
                                                rows="2"
                                                placeholder="Ej. Depositar solo botellas PET limpias y comprimidas..."
                                                class="w-full p-2 bg-slate-50 border border-slate-200 rounded-xl focus:bg-white"
                                                prop:value=move || new_waste_recom.get()
                                                on:input=move |e| new_waste_recom.set(event_target_value(&e))
                                            ></textarea>
                                        </div>

                                        <div>
                                            <label class="font-bold text-slate-700 block mb-1">"URL Fotografía Referencial (Opcional)"</label>
                                            <input
                                                type="text"
                                                placeholder="https://api.ecoalerta.gob.pe/media/uploads/punto-acopio.webp"
                                                class="w-full p-2 bg-slate-50 border border-slate-200 rounded-xl focus:bg-white"
                                                prop:value=move || new_waste_img.get()
                                                on:input=move |e| new_waste_img.set(event_target_value(&e))
                                            />
                                        </div>
                                    </div>

                                    <div class="flex justify-end space-x-2 pt-2 border-t border-slate-100">
                                        <button class="px-3 py-1.5 text-xs text-slate-600 font-semibold" on:click=move |_| show_create_waste_modal.set(false)>
                                            "Cancelar"
                                        </button>
                                        <button
                                            class="px-4 py-2 text-xs font-semibold bg-emerald-600 hover:bg-emerald-700 text-white rounded-xl shadow-xs transition"
                                            disabled=move || is_submitting_action.get()
                                            on:click=submit_create_waste_point
                                        >
                                            {move || if is_submitting_action.get() { "Guardando..." } else if editing_waste_target.get().is_some() { "Guardar Cambios" } else { "Guardar Punto" }}
                                        </button>
                                    </div>
                                </div>
                            </div>
                        }
                    })}

                    // Modal 5: Google Gemini AI Configuration Dialog
                    {move || show_gemini_modal.get().then(|| {
                        let settings_opt = gemini_settings.get();
                        let has_key = settings_opt.as_ref().map(|s| s.has_gemini_key).unwrap_or(false);
                        let masked = settings_opt.as_ref().map(|s| s.masked_key.clone()).unwrap_or_default();
                        let models_list: Vec<GeminiModelInfo> = settings_opt.as_ref().and_then(|s| {
                            if !s.available_models.is_empty() {
                                Some(s.available_models.clone())
                            } else {
                                None
                            }
                        }).unwrap_or_else(|| {
                            vec![
                                GeminiModelInfo {
                                    id: "gemini-3.5-flash".to_string(),
                                    name: "Gemini 3.5 Flash".to_string(),
                                    badge: Some("⭐ Recomendado (Ultra Rápido)".to_string()),
                                    description: Some("Respuesta instantánea para visión por computadora y peritaje ambiental en Cerro de Pasco.".to_string()),
                                },
                                GeminiModelInfo {
                                    id: "gemini-3.8-flash".to_string(),
                                    name: "Gemini 3.8 Flash".to_string(),
                                    badge: Some("🚀 Última Generación".to_string()),
                                    description: Some("Modelo multimodal insignia de Google AI Studio.".to_string()),
                                },
                                GeminiModelInfo {
                                    id: "gemini-3.1-flash-lite".to_string(),
                                    name: "Gemini 3.1 Flash Lite".to_string(),
                                    badge: Some("🍃 Ultraligero".to_string()),
                                    description: Some("Optimizado para bajo consumo de tokens y máxima disponibilidad operativa.".to_string()),
                                },
                                GeminiModelInfo {
                                    id: "gemini-1.5-flash".to_string(),
                                    name: "Gemini 1.5 Flash".to_string(),
                                    badge: Some("⚡ Multimodal Estándar".to_string()),
                                    description: Some("Modelo estándar de Google AI Studio.".to_string()),
                                },
                            ]
                        });

                        view! {
                            <div class="fixed inset-0 z-50 bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4 animate-in fade-in duration-200">
                                <div class="bg-white max-w-xl w-full rounded-2xl border border-slate-200 shadow-2xl overflow-hidden flex flex-col max-h-[90vh]">
                                    // Header
                                    <div class="p-5 border-b border-slate-100 flex items-center justify-between bg-slate-50/60">
                                        <div class="flex items-center space-x-3">
                                            <div class="w-10 h-10 rounded-xl bg-sky-100 text-sky-700 flex items-center justify-center font-bold text-lg shadow-xs">
                                                "⚙️"
                                            </div>
                                            <div>
                                                <h3 class="font-bold text-slate-900 text-base">
                                                    "Configuración del Sistema"
                                                </h3>
                                                <p class="text-xs text-slate-500 font-medium">
                                                    "Google Gemini AI, Seguridad Anti-Bot Turnstile y Parámetros Operativos"
                                                </p>
                                            </div>
                                        </div>
                                        <button
                                            class="p-1.5 rounded-lg text-slate-400 hover:text-slate-600 hover:bg-slate-100 transition"
                                            on:click=move |_| show_gemini_modal.set(false)
                                        >
                                            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                <line x1="18" y1="6" x2="6" y2="18"></line>
                                                <line x1="6" y1="6" x2="18" y2="18"></line>
                                            </svg>
                                        </button>
                                    </div>

                                    // Body
                                    <div class="p-5 overflow-y-auto space-y-4 text-xs">
                                        // Loading indicator if fetching
                                        {move || is_loading_settings.get().then(|| view! {
                                            <div class="p-3 bg-sky-50 rounded-xl border border-sky-200 text-sky-700 flex items-center space-x-2 animate-pulse">
                                                <span class="inline-block w-4 h-4 border-2 border-sky-400 border-t-sky-700 rounded-full animate-spin"></span>
                                                <span class="font-medium">"Cargando configuración actual del servidor..."</span>
                                            </div>
                                        })}

                                        // Status Message Banner (success or error)
                                        {move || gemini_status_msg.get().map(|(is_ok, msg)| {
                                            if is_ok {
                                                view! {
                                                    <div class="p-3 rounded-xl bg-emerald-50 border border-emerald-200 text-emerald-800 flex items-start space-x-2">
                                                        <span class="text-base shrink-0">"✅"</span>
                                                        <p class="font-medium text-xs leading-relaxed">{msg}</p>
                                                    </div>
                                                }.into_view()
                                            } else {
                                                view! {
                                                    <div class="p-3 rounded-xl bg-red-50 border border-red-200 text-red-800 flex items-start space-x-2">
                                                        <span class="text-base shrink-0">"⚠️"</span>
                                                        <p class="font-medium text-xs leading-relaxed">{msg}</p>
                                                    </div>
                                                }.into_view()
                                            }
                                        })}

                                        // Switch Activation Toggle Card
                                        <div class="p-4 rounded-xl border border-slate-200 bg-white shadow-xs flex items-center justify-between gap-3">
                                            <div>
                                                <div class="flex items-center space-x-2">
                                                    <span class="font-bold text-slate-900 text-xs">
                                                        "Activar Motor Inteligente Gemini"
                                                    </span>
                                                    {move || if gemini_enabled_input.get() {
                                                        view! { <span class="px-2 py-0.5 rounded-full bg-emerald-100 text-emerald-700 text-[10px] font-bold">"Activado"</span> }
                                                    } else {
                                                        view! { <span class="px-2 py-0.5 rounded-full bg-slate-200 text-slate-600 text-[10px] font-bold">"Modo Manual"</span> }
                                                    }}
                                                </div>
                                                <p class="text-[11px] text-slate-500 mt-0.5">
                                                    "Analiza fotos automáticamente para prellenar denuncias y detectar residuos menores."
                                                </p>
                                            </div>
                                            <button
                                                type="button"
                                                class=move || {
                                                    if gemini_enabled_input.get() {
                                                        "relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-emerald-600 transition-colors duration-200 ease-in-out focus:outline-hidden"
                                                    } else {
                                                        "relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-slate-200 transition-colors duration-200 ease-in-out focus:outline-hidden"
                                                    }
                                                }
                                                on:click=move |_| gemini_enabled_input.update(|v| *v = !*v)
                                            >
                                                <span
                                                    class=move || {
                                                        if gemini_enabled_input.get() {
                                                            "pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow-xs ring-0 transition duration-200 ease-in-out translate-x-5"
                                                        } else {
                                                            "pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow-xs ring-0 transition duration-200 ease-in-out translate-x-0"
                                                        }
                                                    }
                                                ></span>
                                            </button>
                                        </div>

                                        // API Key Card
                                        <div class="p-4 rounded-xl border border-slate-200 bg-white shadow-xs space-y-3">
                                            <div class="flex items-center justify-between">
                                                <span class="font-bold text-slate-800 text-xs">
                                                    "Google Gemini API Key"
                                                </span>
                                                {if has_key {
                                                    view! {
                                                        <span class="text-[10px] font-bold text-emerald-700 bg-emerald-50 px-2 py-0.5 rounded-md border border-emerald-200">
                                                            {format!("✓ Configurada ({})", masked)}
                                                        </span>
                                                    }.into_view()
                                                } else {
                                                    view! {
                                                        <span class="text-[10px] font-bold text-amber-700 bg-amber-50 px-2 py-0.5 rounded-md border border-amber-200">
                                                            "⚠ No configurada"
                                                        </span>
                                                    }.into_view()
                                                }}
                                            </div>

                                            <div>
                                                <input
                                                    type="password"
                                                    placeholder={if has_key { "Ingresa una nueva clave para reemplazar o deja en blanco..." } else { "Pega tu API Key de Google (AIzaSy...)" }}
                                                    class="w-full p-2.5 bg-slate-50 border border-slate-200 rounded-xl text-xs focus:bg-white focus:outline-hidden focus:ring-2 focus:ring-sky-500 font-mono transition"
                                                    prop:value=move || gemini_key_input.get()
                                                    on:input=move |e| gemini_key_input.set(event_target_value(&e))
                                                />
                                                <div class="flex items-center justify-between mt-1.5 text-[10px] text-slate-500">
                                                    <span>"Clave protegida y almacenada en la base de datos municipal."</span>
                                                    <a
                                                        href="https://aistudio.google.com/app/apikey"
                                                        target="_blank"
                                                        rel="noreferrer"
                                                        class="text-sky-600 hover:underline font-semibold flex items-center space-x-1"
                                                    >
                                                        <span>"Obtener API Key gratuita"</span>
                                                        <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
                                                            <polyline points="15 3 21 3 21 9"></polyline>
                                                            <line x1="10" y1="14" x2="21" y2="3"></line>
                                                        </svg>
                                                    </a>
                                                </div>
                                            </div>
                                        </div>

                                        // Model Selection
                                        <div class="p-4 rounded-xl border border-slate-200 bg-white shadow-xs space-y-2.5">
                                            <label class="font-bold text-slate-800 text-xs block">
                                                "Modelo de Inteligencia Artificial (Google Gemini):"
                                            </label>
                                            <div class="space-y-2">
                                                {models_list.into_iter().map(|m| {
                                                    let m_id_for_click = m.id.clone();
                                                    let m_id_for_card = m.id.clone();
                                                    let m_id_for_dot = m.id.clone();
                                                    view! {
                                                        <div
                                                            class=move || {
                                                                if gemini_model_input.get() == m_id_for_card {
                                                                    "p-3 rounded-xl border-2 border-sky-500 bg-sky-50/50 cursor-pointer transition shadow-xs"
                                                                } else {
                                                                    "p-3 rounded-xl border border-slate-200 hover:border-slate-300 bg-white cursor-pointer transition"
                                                                }
                                                            }
                                                            on:click={
                                                                let m_id_click = m_id_for_click.clone();
                                                                move |_| gemini_model_input.set(m_id_click.clone())
                                                            }
                                                        >
                                                            <div class="flex items-center justify-between">
                                                                <div class="flex items-center space-x-2">
                                                                    <span class=move || {
                                                                        if gemini_model_input.get() == m_id_for_dot {
                                                                            "w-3.5 h-3.5 rounded-full border-4 border-sky-600 bg-white shrink-0"
                                                                        } else {
                                                                            "w-3.5 h-3.5 rounded-full border-2 border-slate-300 bg-white shrink-0"
                                                                        }
                                                                    }></span>
                                                                    <span class="font-bold text-slate-900 text-xs">{m.name}</span>
                                                                </div>
                                                                {m.badge.map(|b| view! {
                                                                    <span class="text-[10px] font-bold px-2 py-0.5 rounded-full bg-sky-100 text-sky-800">
                                                                        {b}
                                                                    </span>
                                                                })}
                                                            </div>
                                                            {m.description.map(|d| view! {
                                                                <p class="text-[11px] text-slate-500 mt-1 pl-5.5 leading-relaxed">
                                                                    {d}
                                                                </p>
                                                            })}
                                                        </div>
                                                    }
                                                }).collect_view()}
                                            </div>
                                        </div>

                                        // Test Handshake Box
                                        <div class="p-3.5 rounded-xl border border-dashed border-slate-300 bg-slate-50/50 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3">
                                            <div>
                                                <h4 class="font-bold text-slate-800 text-xs">"Diagnóstico de Conectividad"</h4>
                                                <p class="text-[11px] text-slate-500">
                                                    "Verifica la validez de la API Key y la latencia con Google AI Studio."
                                                </p>
                                            </div>
                                            <button
                                                class="px-3.5 py-2 text-xs font-semibold bg-white hover:bg-slate-100 text-slate-800 border border-slate-300 rounded-xl shadow-xs transition flex items-center space-x-1.5 shrink-0"
                                                disabled=move || is_testing_gemini.get() || is_saving_settings.get()
                                                on:click=test_gemini_connection
                                            >
                                                {move || if is_testing_gemini.get() {
                                                    view! {
                                                        <div class="flex items-center space-x-1.5">
                                                            <span class="inline-block w-3.5 h-3.5 border-2 border-slate-400 border-t-slate-800 rounded-full animate-spin"></span>
                                                            <span>"Comprobando..."</span>
                                                        </div>
                                                    }.into_view()
                                                } else {
                                                    view! {
                                                        <div class="flex items-center space-x-1.5">
                                                            <span>"⚡ Probar Conexión"</span>
                                                        </div>
                                                    }.into_view()
                                                }}
                                            </button>
                                        </div>

                                        // Test result detail card if available
                                        {move || gemini_test_result.get().map(|res| {
                                            if res.success {
                                                view! {
                                                    <div class="p-3.5 rounded-xl bg-slate-900 text-white space-y-1.5 font-mono text-[11px]">
                                                        <div class="flex items-center justify-between text-emerald-400 font-bold">
                                                            <span>"HANDSHAKE: " {res.status.unwrap_or_else(|| "OPERATIVO".to_string())}</span>
                                                            <span>{res.latency_ms.map(|l| format!("{} ms", l)).unwrap_or_default()}</span>
                                                        </div>
                                                        <div class="text-slate-300">
                                                            "Modelo probado: " {res.model.unwrap_or_default()}
                                                        </div>
                                                        {res.quota_tier.map(|q| view! {
                                                            <div class="text-slate-400 text-[10px]">"Nivel de cuota: " {q}</div>
                                                        })}
                                                        {res.message.map(|msg| view! {
                                                            <div class="text-slate-300 text-[10px] italic border-t border-slate-800 pt-1 mt-1">
                                                                {msg}
                                                            </div>
                                                        })}
                                                    </div>
                                                }.into_view()
                                            } else {
                                                view! {
                                                    <div class="p-3.5 rounded-xl bg-red-950 text-red-200 text-[11px] font-mono space-y-1 border border-red-800">
                                                        <p class="font-bold text-red-400">"ERROR EN HANDSHAKE:"</p>
                                                        <p>{res.error.or(res.message).unwrap_or_else(|| "Fallo en la prueba de conexión con Gemini".to_string())}</p>
                                                    </div>
                                                }.into_view()
                                            }
                                        })}

                                        // ==================== CLOUDFLARE TURNSTILE SECTION ====================
                                        {
                                            let has_secret = settings_opt.as_ref().and_then(|s| s.has_turnstile_secret).unwrap_or(false);
                                            let masked_secret = settings_opt.as_ref().and_then(|s| s.turnstile_secret_masked.clone()).unwrap_or_default();

                                            view! {
                                                <div class="p-4 rounded-xl border border-slate-200 bg-white shadow-xs space-y-3">
                                                    <div class="flex items-center justify-between">
                                                        <div class="flex items-center space-x-2">
                                                            <div class="w-7 h-7 rounded-lg bg-amber-50 text-amber-700 flex items-center justify-center font-bold text-sm">
                                                                "🛡️"
                                                            </div>
                                                            <div>
                                                                <span class="font-bold text-slate-800 text-xs block">
                                                                    "Seguridad Anti-Bot (Cloudflare Turnstile)"
                                                                </span>
                                                                <span class="text-[10px] text-slate-400">
                                                                    "Protege el registro de usuarios ciudadanos contra spam y ataques"
                                                                </span>
                                                            </div>
                                                        </div>
                                                        <div class="flex items-center space-x-2">
                                                            {move || if turnstile_enabled_input.get() {
                                                                view! { <span class="px-2 py-0.5 rounded-full bg-emerald-100 text-emerald-700 text-[10px] font-bold">"Activado"</span> }
                                                            } else {
                                                                view! { <span class="px-2 py-0.5 rounded-full bg-slate-200 text-slate-600 text-[10px] font-bold">"Desactivado"</span> }
                                                            }}
                                                            <button
                                                                type="button"
                                                                class=move || {
                                                                    if turnstile_enabled_input.get() {
                                                                        "relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-emerald-600 transition-colors duration-200 ease-in-out focus:outline-hidden"
                                                                    } else {
                                                                        "relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-slate-200 transition-colors duration-200 ease-in-out focus:outline-hidden"
                                                                    }
                                                                }
                                                                on:click=move |_| turnstile_enabled_input.update(|v| *v = !*v)
                                                            >
                                                                <span
                                                                    class=move || {
                                                                        if turnstile_enabled_input.get() {
                                                                            "pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow-xs ring-0 transition duration-200 ease-in-out translate-x-4"
                                                                        } else {
                                                                            "pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow-xs ring-0 transition duration-200 ease-in-out translate-x-0"
                                                                        }
                                                                    }
                                                                ></span>
                                                            </button>
                                                        </div>
                                                    </div>

                                                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-1">
                                                        <div>
                                                            <label class="block text-[10px] font-bold text-slate-500 uppercase tracking-wider mb-1">
                                                                "Site Key (Pública)"
                                                            </label>
                                                            <input
                                                                type="text"
                                                                placeholder="1x00000000000000000000AA"
                                                                class="w-full p-2 bg-slate-50 border border-slate-200 rounded-xl text-xs font-mono focus:bg-white focus:outline-hidden focus:ring-2 focus:ring-amber-500"
                                                                prop:value=move || turnstile_sitekey_input.get()
                                                                on:input=move |e| turnstile_sitekey_input.set(event_target_value(&e))
                                                            />
                                                        </div>
                                                        <div>
                                                            <div class="flex items-center justify-between mb-1">
                                                                <label class="block text-[10px] font-bold text-slate-500 uppercase tracking-wider">
                                                                    "Secret Key (Privada)"
                                                                </label>
                                                                {if has_secret {
                                                                    view! {
                                                                        <span class="text-[9px] font-bold text-emerald-700 bg-emerald-50 px-1.5 py-0.5 rounded border border-emerald-200">
                                                                            {format!("✓ Configurada ({})", masked_secret)}
                                                                        </span>
                                                                    }.into_view()
                                                                } else {
                                                                    view! {
                                                                        <span class="text-[9px] font-bold text-slate-500 bg-slate-100 px-1.5 py-0.5 rounded">
                                                                            "Test Keys (Cloudflare)"
                                                                        </span>
                                                                    }.into_view()
                                                                }}
                                                            </div>
                                                            <input
                                                                type="password"
                                                                placeholder={if has_secret { "Reemplazar clave secreta..." } else { "Pega la Secret Key de Cloudflare" }}
                                                                class="w-full p-2 bg-slate-50 border border-slate-200 rounded-xl text-xs font-mono focus:bg-white focus:outline-hidden focus:ring-2 focus:ring-amber-500"
                                                                prop:value=move || turnstile_secret_input.get()
                                                                on:input=move |e| turnstile_secret_input.set(event_target_value(&e))
                                                            />
                                                        </div>
                                                    </div>

                                                    <div class="text-[10px] text-slate-400 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-1 pt-1 border-t border-slate-100">
                                                        <span>"Claves por defecto: Test universal de Cloudflare (siempre aprobadas en desarrollo)."</span>
                                                        <a
                                                            href="https://dash.cloudflare.com/?to=/:account/turnstile"
                                                            target="_blank"
                                                            rel="noreferrer"
                                                            class="text-amber-600 hover:underline font-semibold"
                                                        >
                                                            "Abrir Cloudflare Turnstile →"
                                                        </a>
                                                    </div>
                                                </div>
                                            }
                                        }
                                    </div>

                                    // Footer
                                    <div class="p-4 border-t border-slate-100 bg-slate-50 flex items-center justify-end space-x-2">
                                        <button
                                            class="px-4 py-2 text-xs font-semibold text-slate-600 hover:text-slate-800 transition"
                                            on:click=move |_| show_gemini_modal.set(false)
                                        >
                                            "Cerrar"
                                        </button>
                                        <button
                                            class="px-5 py-2 text-xs font-bold bg-sky-600 hover:bg-sky-700 text-white rounded-xl shadow-xs transition flex items-center space-x-2"
                                            disabled=move || is_saving_settings.get() || is_testing_gemini.get()
                                            on:click=save_gemini_settings
                                        >
                                            {move || if is_saving_settings.get() {
                                                view! {
                                                    <div class="flex items-center space-x-1.5">
                                                        <span class="inline-block w-3.5 h-3.5 border-2 border-white/40 border-t-white rounded-full animate-spin"></span>
                                                        <span>"Guardando..."</span>
                                                    </div>
                                                }.into_view()
                                            } else {
                                                view! {
                                                    <div class="flex items-center space-x-1.5">
                                                        <span>"Guardar Configuración"</span>
                                                    </div>
                                                }.into_view()
                                            }}
                                        </button>
                                    </div>
                                </div>
                            </div>
                        }.into_view()
                    })}

                    // MODAL 6: ASIGNAR CUADRILLA / INSPECTOR
                    {move || assign_squad_target.get().map(|target| {
                        view! {
                            <div class="fixed inset-0 z-50 bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4 animate-in fade-in duration-200">
                                <div class="bg-white max-w-lg w-full rounded-2xl border border-slate-200 shadow-2xl p-6 space-y-4">
                                    <div class="flex items-start justify-between">
                                        <div class="flex items-center space-x-3">
                                            <div class="w-10 h-10 rounded-xl bg-amber-100 text-amber-700 flex items-center justify-center text-lg font-bold shadow-xs">
                                                "👷"
                                            </div>
                                            <div>
                                                <h3 class="font-bold text-slate-900 text-base leading-snug">
                                                    "Asignar Cuadrilla / Inspector"
                                                </h3>
                                                <p class="text-xs text-slate-500 font-medium">
                                                    "El caso pasará al estado oficial \"En Inspección\""
                                                </p>
                                            </div>
                                        </div>
                                        <button
                                            class="p-1.5 rounded-lg text-slate-400 hover:text-slate-600 hover:bg-slate-100 transition"
                                            on:click=move |_| assign_squad_target.set(None)
                                        >
                                            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                <line x1="18" y1="6" x2="6" y2="18"></line>
                                                <line x1="6" y1="6" x2="18" y2="18"></line>
                                            </svg>
                                        </button>
                                    </div>

                                    <div class="p-3 bg-slate-50 rounded-xl border border-slate-200/80 text-xs">
                                        <p class="font-bold text-slate-900">{target.title.clone()}</p>
                                        <p class="text-slate-500 text-[11px] mt-0.5 truncate">{target.address.clone()}</p>
                                    </div>

                                    {move || action_error.get().map(|e| view! {
                                        <div class="p-2.5 bg-red-50 text-red-700 text-xs rounded-xl border border-red-200">
                                            {e}
                                        </div>
                                    })}

                                    <div class="space-y-1">
                                        <label class="text-[11px] font-bold text-slate-700 uppercase tracking-wider block">
                                            "Cuadrilla o Unidad Municipal:"
                                        </label>
                                        <select
                                            class="w-full p-2.5 text-xs bg-slate-50 border border-slate-200 rounded-xl font-medium focus:bg-white focus:outline-hidden focus:ring-2 focus:ring-slate-900"
                                            prop:value=move || selected_squad.get()
                                            on:change=move |e| selected_squad.set(event_target_value(&e))
                                        >
                                            <option value="Cuadrilla de Limpieza y Residuos Sólidos">"Cuadrilla de Limpieza y Residuos Sólidos"</option>
                                            <option value="Equipo de Fiscalización Ambiental y Salubridad">"Equipo de Fiscalización Ambiental y Salubridad"</option>
                                            <option value="Serenazgo y Patrullaje Ecológico">"Serenazgo y Patrullaje Ecológico"</option>
                                            <option value="Unidad de Mantenimiento y Parques">"Unidad de Mantenimiento y Parques"</option>
                                            <option value="Subgerencia de Gestión Ambiental (OEFA)">"Subgerencia de Gestión Ambiental (OEFA)"</option>
                                        </select>
                                    </div>

                                    <div class="space-y-1">
                                        <label class="text-[11px] font-bold text-slate-700 uppercase tracking-wider block">
                                            "Inspector o Responsable de Campo:"
                                        </label>
                                        <input
                                            type="text"
                                            placeholder="Ej. Ing. Carlos Mendoza"
                                            class="w-full p-2.5 text-xs bg-slate-50 border border-slate-200 rounded-xl focus:bg-white focus:outline-hidden focus:ring-2 focus:ring-slate-900 transition"
                                            prop:value=move || inspector_name.get()
                                            on:input=move |e| inspector_name.set(event_target_value(&e))
                                        />
                                    </div>

                                    <div class="space-y-1">
                                        <label class="text-[11px] font-bold text-slate-700 uppercase tracking-wider block">
                                            "Instrucciones u Órdenes de Intervención:"
                                        </label>
                                        <textarea
                                            rows="2"
                                            placeholder="Detalla las instrucciones operativas..."
                                            class="w-full p-2.5 text-xs bg-slate-50 border border-slate-200 rounded-xl focus:bg-white focus:outline-hidden focus:ring-2 focus:ring-slate-900 transition"
                                            prop:value=move || squad_notes.get()
                                            on:input=move |e| squad_notes.set(event_target_value(&e))
                                        ></textarea>
                                    </div>

                                    <div class="flex justify-end space-x-2 pt-2 border-t border-slate-100">
                                        <button
                                            class="px-3 py-1.5 text-xs text-slate-600 font-semibold"
                                            on:click=move |_| assign_squad_target.set(None)
                                        >
                                            "Cancelar"
                                        </button>
                                        <button
                                            class="px-4 py-2 text-xs font-semibold bg-amber-600 hover:bg-amber-700 text-white rounded-xl shadow-xs transition"
                                            disabled=move || is_submitting_action.get()
                                            on:click=submit_assign_squad
                                        >
                                            {move || if is_submitting_action.get() { "Asignando..." } else { "Asignar Cuadrilla" }}
                                        </button>
                                    </div>
                                </div>
                            </div>
                        }
                    })}

                    // MODAL 7: BITÁCORA Y AUDITORÍA DE CASO
                    {move || history_target.get().map(|target| {
                        let history_items = target.parsed_history();

                        view! {
                            <div class="fixed inset-0 z-50 bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4 animate-in fade-in duration-200">
                                <div class="bg-white max-w-md w-full rounded-2xl border border-slate-200 shadow-2xl p-6 space-y-4 max-h-[85vh] flex flex-col">
                                    <div class="flex items-start justify-between border-b border-slate-100 pb-3 shrink-0">
                                        <div class="flex items-center space-x-3">
                                            <div class="w-10 h-10 rounded-xl bg-purple-100 text-purple-700 flex items-center justify-center text-lg font-bold shadow-xs">
                                                "📜"
                                            </div>
                                            <div>
                                                <h3 class="font-bold text-slate-900 text-base leading-snug">
                                                    "Bitácora y Auditoría de Caso"
                                                </h3>
                                                <p class="text-xs text-slate-500 font-medium">
                                                    "Línea de tiempo y trazabilidad de intervenciones"
                                                </p>
                                            </div>
                                        </div>
                                        <button
                                            class="p-1.5 rounded-lg text-slate-400 hover:text-slate-600 hover:bg-slate-100 transition"
                                            on:click=move |_| history_target.set(None)
                                        >
                                            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                <line x1="18" y1="6" x2="6" y2="18"></line>
                                                <line x1="6" y1="6" x2="18" y2="18"></line>
                                            </svg>
                                        </button>
                                    </div>

                                    <div class="p-3 bg-slate-50 rounded-xl border border-slate-200/80 text-xs shrink-0">
                                        <p class="font-bold text-slate-900 leading-snug">{target.title.clone()}</p>
                                        <p class="text-slate-500 text-[11px] mt-0.5 truncate">{target.address.clone()}</p>
                                    </div>

                                    <div class="overflow-y-auto space-y-3 pr-1 text-xs flex-1">
                                        {if history_items.is_empty() {
                                            view! {
                                                <div class="p-6 text-center text-slate-400 space-y-1">
                                                    <p class="text-2xl">"🌱"</p>
                                                    <p class="font-medium text-xs">"No hay historial de cambios registrado aún."</p>
                                                    <p class="text-[11px] text-slate-400">"El reporte se encuentra en su estado inicial de recepción."</p>
                                                </div>
                                            }.into_view()
                                        } else {
                                            let total_items = history_items.len();
                                            history_items.into_iter().enumerate().map(|(idx, (date_str, action_str))| {
                                                let (icon, bg_color, text_color) = if action_str.contains("Reportado") || action_str.contains("creado") {
                                                    ("📢", "bg-amber-100", "text-amber-800")
                                                } else if action_str.contains("Cuadrilla") || action_str.contains("Inspección") || action_str.contains("Asignada") {
                                                    ("👷", "bg-purple-100", "text-purple-800")
                                                } else if action_str.contains("Transferido") || action_str.contains("Jurisdicción") {
                                                    ("🔄", "bg-indigo-100", "text-indigo-800")
                                                } else if action_str.contains("Solucionado") || action_str.contains("Resuelto") || action_str.contains("Atendido") {
                                                    ("✅", "bg-emerald-100", "text-emerald-800")
                                                } else if action_str.contains("Descartado") {
                                                    ("❌", "bg-slate-200", "text-slate-700")
                                                } else {
                                                    ("ℹ️", "bg-sky-100", "text-sky-800")
                                                };

                                                let is_last = idx == total_items - 1;

                                                view! {
                                                    <div class="flex items-start space-x-3">
                                                        <div class="flex flex-col items-center">
                                                            <div class={format!("w-7 h-7 rounded-full flex items-center justify-center text-xs shrink-0 {} {}", bg_color, text_color)}>
                                                                {icon}
                                                            </div>
                                                            {(!is_last).then(|| view! {
                                                                <div class="w-0.5 h-8 bg-slate-200 my-0.5"></div>
                                                            })}
                                                        </div>
                                                        <div class="flex-1 pb-2">
                                                            <div class="flex items-center justify-between text-[10px] text-slate-400 font-mono">
                                                                <span>{date_str}</span>
                                                            </div>
                                                            <p class="font-semibold text-slate-800 text-xs mt-0.5 leading-relaxed">
                                                                {action_str}
                                                            </p>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view()
                                        }}
                                    </div>

                                    <div class="flex justify-end pt-2 border-t border-slate-100 shrink-0">
                                        <button
                                            class="px-4 py-2 text-xs font-semibold bg-slate-900 hover:bg-slate-800 text-white rounded-xl shadow-xs transition"
                                            on:click=move |_| history_target.set(None)
                                        >
                                            "Cerrar"
                                        </button>
                                    </div>
                                </div>
                            </div>
                        }
                    })}
                </div>
            }.into_view()
        }
    })}
    }
}
