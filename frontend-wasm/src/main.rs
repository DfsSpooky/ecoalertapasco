mod api;
mod components;
mod geo;
mod models;
mod storage;

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use leptos::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use api::{sync_js_api_base, sync_url, ws_map_url};
use components::{
    alert_detail::AlertDetail,
    collector_panel::CollectorPanel,
    auth_dialog::AuthDialog,
    authority_dashboard::AuthorityDashboard,
    emergency_banner::EmergencyBanner,
    help_guide_modal::HelpGuideModal,
    inspection_report_modal::InspectionReportModal,
    map::MapView,
    report_dialog::ReportDialog,
    sidebar::Sidebar,
    transparency_portal::TransparencyPortal,
    user_profile_modal::UserProfileModal,
    waste_detail::WasteDetail,
    waste_list_modal::WasteListModal,
};
use models::{
    CollectionRoute, EcoAlert, TruckRun, EcoCategory, EcoDistrict, EcoSeverity, SyncResponse, SyncStatus, WastePoint,
    WsServerMessage,
};
use storage::{clear_auth_session, load_auth_session};

struct ActiveWsConnection {
    socket: web_sys::WebSocket,
    _onopen: Closure<dyn FnMut(web_sys::Event)>,
    _onmessage: Closure<dyn FnMut(web_sys::MessageEvent)>,
    _onclose: Closure<dyn FnMut(web_sys::CloseEvent)>,
}

impl ActiveWsConnection {
    fn detach_and_close(&self) {
        self.socket.set_onopen(None);
        self.socket.set_onmessage(None);
        self.socket.set_onclose(None);
        let _ = self.socket.close();
    }
}

#[component]
fn App() -> impl IntoView {
    // Production state: initialized strictly empty (zero mocks or assumptions; truth comes from server)
    let alerts = create_rw_signal(Vec::<EcoAlert>::new());
    let selected_categories = create_rw_signal(HashSet::from_iter(EcoCategory::all().iter().copied()));
    let selected_severities = create_rw_signal(HashSet::from_iter(EcoSeverity::all().iter().copied()));
    let selected_district = create_rw_signal(None::<EcoDistrict>);
    let search_query = create_rw_signal("".to_string());
    let selected_alert_id = create_rw_signal(None::<String>);
    let is_report_mode = create_rw_signal(false);
    let report_center = create_rw_signal((-10.6675, -76.2567));
    let show_new_report_dialog = create_rw_signal(false);
    let show_mobile_report_options = create_rw_signal(false);

    // Advanced filters
    let status_filter = create_rw_signal("todos".to_string());
    let days_filter = create_rw_signal(0u32);

    // Inspection Report & Help Guide Modals
    let inspection_target = create_rw_signal(None::<EcoAlert>);
    let show_help_guide = create_rw_signal(false);

    // Live critical alert early-warning banner (triggered ONLY by real incoming critical reports)
    let active_emergency_alert = create_rw_signal(None::<EcoAlert>);

    // Solid waste & truck routes layer state
    let waste_points = create_rw_signal(Vec::<WastePoint>::new());
    let collection_routes = create_rw_signal(Vec::<CollectionRoute>::new());
    let truck_runs = create_rw_signal(Vec::<TruckRun>::new());
    let show_collector_panel = create_rw_signal(false);
    let show_waste_layer = create_rw_signal(false);
    let selected_waste_point_id = create_rw_signal(None::<String>);
    let show_waste_list_modal = create_rw_signal(false);

    // Authentication state
    let current_user = create_rw_signal(load_auth_session());
    let show_auth_dialog = create_rw_signal(false);

    // Transparency Portal & Authority Dashboard & Profile Modals
    let show_transparency_portal = create_rw_signal(false);
    let show_authority_dashboard = create_rw_signal(false);
    let show_user_profile = create_rw_signal(false);

    // Real-Time WebSocket + Fallback Telemetry State
    let sync_status = create_rw_signal(SyncStatus::Connecting);
    let ws_connected = create_rw_signal(false);
    let ws_failures = create_rw_signal(0u32);
    let next_ws_retry_ms = create_rw_signal(0.0f64);
    let last_ping_ms = create_rw_signal(0.0f64);
    let last_ws_auth_token = create_rw_signal(None::<String>);
    let has_completed_initial_sync = create_rw_signal(false);

    // HTTP ETag Safety-Net Fallback State (used ONLY if a corporate firewall blocks WebSockets)
    let last_etag = create_rw_signal(None::<String>);
    let http_fallback_in_flight = create_rw_signal(false);
    let last_http_fallback_ms = create_rw_signal(0.0f64);

    let ws_holder: Rc<RefCell<Option<ActiveWsConnection>>> = Rc::new(RefCell::new(None));

    // Responsive sidebar state (starts collapsed on mobile screens < 768px, open on desktop)
    let is_sidebar_collapsed = create_rw_signal({
        if let Some(w) = web_sys::window() {
            if let Ok(width_val) = w.inner_width() {
                width_val.as_f64().map(|w| w < 768.0).unwrap_or(false)
            } else {
                false
            }
        } else {
            false
        }
    });

    // Expose resolved production API base URL to JS helpers (PDF QR verification) & initialize Web Notifications
    create_effect(move |_| {
        sync_js_api_base();
        let _ = js_sys::eval("if (window.ecoNotify && typeof window.ecoNotify.requestPermission === 'function') { window.ecoNotify.requestPermission(); }");
    });

    // Security guard: Ensure authority dashboard is closed if user logs out or lacks authority permissions
    create_effect(move |_| {
        let is_authority = current_user.get().map(|u| u.is_authority()).unwrap_or(false);
        if !is_authority && show_authority_dashboard.get() {
            show_authority_dashboard.set(false);
        }
    });

    // Authoritative security sync: Validate token and refresh is_staff role from backend on startup
    create_effect(move |_| {
        if let Some(user) = current_user.get_untracked() {
            let token = user.token.clone();
            spawn_local(async move {
                let req = gloo_net::http::Request::get(&crate::api::profile_url())
                    .header("Authorization", &format!("Token {}", token))
                    .send()
                    .await;

                if let Ok(resp) = req {
                    if resp.ok() {
                        if let Ok(profile_data) = resp.json::<serde_json::Value>().await {
                            let backend_is_staff = profile_data.get("is_staff").and_then(|v| v.as_bool()).unwrap_or(false);
                            let backend_is_collector = profile_data.get("is_collector").and_then(|v| v.as_bool()).unwrap_or(false);
                            let needs_update = current_user.with_untracked(|u_opt| {
                                u_opt.as_ref().map(|u| u.is_staff != backend_is_staff || u.is_collector != backend_is_collector).unwrap_or(false)
                            });
                            if needs_update {
                                current_user.update(|u_opt| {
                                    if let Some(ref mut u) = u_opt {
                                        u.is_staff = backend_is_staff;
                                        u.is_collector = backend_is_collector;
                                        crate::storage::save_auth_session(u);
                                    }
                                });
                            }
                        }
                    } else if resp.status() == 401 {
                        crate::storage::clear_auth_session();
                        current_user.set(None);
                    }
                }
            });
        }
    });

    // Safety-net HTTP conditional ETag sync (dormant while WebSocket is connected)
    let run_http_fallback_sync: Rc<dyn Fn()> = Rc::new(move || {
        if ws_connected.get_untracked() || http_fallback_in_flight.get_untracked() {
            return;
        }
        http_fallback_in_flight.set(true);
        last_http_fallback_ms.set(js_sys::Date::now());

        let current_token = current_user.get_untracked().map(|u| u.token);
        let etag_arg = last_etag.get_untracked();

        spawn_local(async move {
            let mut req = gloo_net::http::Request::get(&sync_url(etag_arg.as_deref()));
            if let Some(ref tok) = current_token {
                req = req.header("Authorization", &format!("Token {}", tok));
            }
            if let Ok(resp) = req.send().await {
                if resp.status() == 304 {
                    has_completed_initial_sync.set(true);
                    if !ws_connected.get_untracked() {
                        sync_status.set(SyncStatus::FallbackHttp);
                    }
                } else if resp.ok() {
                    if let Ok(sync_data) = resp.json::<SyncResponse>().await {
                        if alerts.with_untracked(|cur| cur != &sync_data.alerts) {
                            alerts.set(sync_data.alerts);
                        }
                        if waste_points.with_untracked(|cur| cur != &sync_data.waste_points) {
                            waste_points.set(sync_data.waste_points);
                        }
                        if collection_routes.with_untracked(|cur| cur != &sync_data.routes) {
                            collection_routes.set(sync_data.routes);
                        }
                        if truck_runs.with_untracked(|cur| cur != &sync_data.truck_runs) {
                            truck_runs.set(sync_data.truck_runs);
                        }
                        last_etag.set(Some(sync_data.etag));
                        has_completed_initial_sync.set(true);
                        if !ws_connected.get_untracked() {
                            sync_status.set(SyncStatus::FallbackHttp);
                        }
                    }
                } else if resp.status() == 401 && current_token.is_some() {
                    clear_auth_session();
                    last_etag.set(None);
                    current_user.set(None);
                }
            }
            http_fallback_in_flight.set(false);
        });
    });

    // Primary Real-Time Full-Duplex WebSocket Connection Manager
    let connect_ws: Rc<dyn Fn(bool)> = {
        let ws_holder = Rc::clone(&ws_holder);
        Rc::new(move |force_reconnect: bool| {
            if let Some(win) = web_sys::window() {
                if !win.navigator().on_line() {
                    ws_connected.set(false);
                    sync_status.set(SyncStatus::Offline);
                    return;
                }
            }

            {
                let mut guard = ws_holder.borrow_mut();
                if let Some(ref existing) = *guard {
                    let state = existing.socket.ready_state();
                    if !force_reconnect
                        && (state == web_sys::WebSocket::OPEN || state == web_sys::WebSocket::CONNECTING)
                    {
                        return;
                    }
                    existing.detach_and_close();
                }
                *guard = None;
            }

            let token_opt = current_user.get_untracked().map(|u| u.token);
            let url = ws_map_url(token_opt.as_deref());

            let cur_stat = sync_status.get_untracked();
            if cur_stat != SyncStatus::FallbackHttp {
                sync_status.set(SyncStatus::Connecting);
            }

            let socket = match web_sys::WebSocket::new(&url) {
                Ok(ws) => ws,
                Err(_) => {
                    ws_connected.set(false);
                    let fails = (ws_failures.get_untracked() + 1).min(5);
                    ws_failures.set(fails);
                    next_ws_retry_ms.set(js_sys::Date::now() + 3000.0);
                    sync_status.set(SyncStatus::Offline);
                    return;
                }
            };

            let token_on_open = token_opt.clone();
            let onopen = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                ws_connected.set(true);
                ws_failures.set(0);
                next_ws_retry_ms.set(0.0);
                last_ping_ms.set(js_sys::Date::now());
                last_ws_auth_token.set(token_on_open.clone());
                sync_status.set(SyncStatus::RealtimeWs);
            });

            let onmessage = Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |ev: web_sys::MessageEvent| {
                let Some(raw_text) = ev.data().as_string() else {
                    return;
                };
                let Ok(msg) = serde_json::from_str::<WsServerMessage>(&raw_text) else {
                    return;
                };

                match msg {
                    WsServerMessage::InitialSnapshot {
                        alerts: snap_alerts,
                        waste_points: snap_wp,
                        routes: snap_routes,
                        truck_runs: snap_runs,
                        ..
                    } => {
                        if truck_runs.with_untracked(|cur| cur != &snap_runs) {
                            truck_runs.set(snap_runs);
                        }
                        if collection_routes.with_untracked(|cur| cur != &snap_routes) {
                            collection_routes.set(snap_routes);
                        }
                        if alerts.with_untracked(|cur| cur != &snap_alerts) {
                            alerts.set(snap_alerts);
                        }
                        if waste_points.with_untracked(|cur| cur != &snap_wp) {
                            waste_points.set(snap_wp);
                        }
                        last_etag.set(None);
                        has_completed_initial_sync.set(true);
                        ws_connected.set(true);
                        sync_status.set(SyncStatus::RealtimeWs);
                    }
                    WsServerMessage::AlertUpserted { alert: incoming } => {
                        let is_new = alerts.with_untracked(|list| !list.iter().any(|a| a.id == incoming.id));
                        if is_new {
                            let is_own = current_user.with_untracked(|u| {
                                u.as_ref().map(|cur| {
                                    incoming.reporter_name.as_deref() == Some(&cur.username)
                                        || incoming.reporter_name.as_deref() == cur.full_name.as_deref()
                                }).unwrap_or(false)
                            });
                            if has_completed_initial_sync.get_untracked()
                                && incoming.severity == EcoSeverity::Critico
                                && incoming.status == "pending"
                                && !is_own
                            {
                                active_emergency_alert.set(Some(incoming.clone()));
                                if let (Ok(t_json), Ok(d_json)) = (
                                    serde_json::to_string(&incoming.title),
                                    serde_json::to_string(&incoming.address),
                                ) {
                                    let _ = js_sys::eval(&format!(
                                        "if (window.ecoNotify && typeof window.ecoNotify.send === 'function') {{ window.ecoNotify.send({}, {}); }}",
                                        t_json, d_json
                                    ));
                                }
                            }
                            alerts.update(|list| list.insert(0, incoming));
                        } else {
                            let changed = alerts.with_untracked(|list| {
                                list.iter()
                                    .find(|a| a.id == incoming.id)
                                    .map(|cur| cur != &incoming)
                                    .unwrap_or(false)
                            });
                            if changed {
                                alerts.update(|list| {
                                    if let Some(existing) = list.iter_mut().find(|a| a.id == incoming.id) {
                                        *existing = incoming;
                                    }
                                });
                            }
                        }
                    }
                    WsServerMessage::AlertDeleted { id } => {
                        if selected_alert_id.get_untracked().as_deref() == Some(id.as_str()) {
                            selected_alert_id.set(None);
                        }
                        alerts.update(|list| list.retain(|a| a.id != id));
                    }
                    WsServerMessage::WastePointUpserted { waste_point: wp } => {
                        let wid = wp.id_string();
                        let exists = waste_points.with_untracked(|list| list.iter().any(|p| p.id_string() == wid));
                        if !exists {
                            waste_points.update(|list| list.push(wp));
                        } else {
                            let changed = waste_points.with_untracked(|list| {
                                list.iter()
                                    .find(|p| p.id_string() == wid)
                                    .map(|cur| cur != &wp)
                                    .unwrap_or(false)
                            });
                            if changed {
                                waste_points.update(|list| {
                                    if let Some(existing) = list.iter_mut().find(|p| p.id_string() == wid) {
                                        *existing = wp;
                                    }
                                });
                            }
                        }
                    }
                    WsServerMessage::WastePointDeleted { id } => {
                        if selected_waste_point_id.get_untracked().as_deref() == Some(id.as_str()) {
                            selected_waste_point_id.set(None);
                        }
                        waste_points.update(|list| list.retain(|p| p.id_string() != id));
                    }
                    WsServerMessage::RouteUpserted { route } => {
                        let rid = route.id_string();
                        let changed = collection_routes.with_untracked(|list| {
                            list.iter().find(|r| r.id_string() == rid).map(|cur| cur != &route).unwrap_or(true)
                        });
                        if changed {
                            collection_routes.update(|list| {
                                if let Some(existing) = list.iter_mut().find(|r| r.id_string() == rid) {
                                    *existing = route;
                                } else {
                                    list.push(route);
                                }
                            });
                        }
                    }
                    WsServerMessage::RouteDeleted { id } => {
                        if collection_routes.with_untracked(|list| list.iter().any(|r| r.id_string() == id)) {
                            collection_routes.update(|list| list.retain(|r| r.id_string() != id));
                        }
                    }
                    WsServerMessage::TruckRunUpserted { run } => {
                        truck_runs.update(|list| {
                            if let Some(existing) = list.iter_mut().find(|r| r.id == run.id) {
                                // Conservar la última posición si el evento de estado llega sin ella
                                let mut merged = run;
                                if merged.last_latitude.is_none() {
                                    merged.last_latitude = existing.last_latitude;
                                    merged.last_longitude = existing.last_longitude;
                                    merged.last_heading = existing.last_heading;
                                    merged.last_position_at = existing.last_position_at.clone();
                                }
                                *existing = merged;
                            } else {
                                list.push(run);
                            }
                        });
                    }
                    WsServerMessage::TruckRunEnded { id } => {
                        if truck_runs.with_untracked(|list| list.iter().any(|r| r.id.to_string() == id)) {
                            truck_runs.update(|list| list.retain(|r| r.id.to_string() != id));
                        }
                    }
                    WsServerMessage::TruckPosition { position } => {
                        let known = truck_runs.with_untracked(|list| list.iter().any(|r| r.id == position.run_id));
                        if known {
                            truck_runs.update(|list| {
                                if let Some(r) = list.iter_mut().find(|r| r.id == position.run_id) {
                                    r.last_latitude = Some(position.latitude);
                                    r.last_longitude = Some(position.longitude);
                                    r.last_heading = position.heading;
                                    r.last_speed_kmh = position.speed_kmh;
                                    r.last_position_at = position.at.clone();
                                }
                            });
                        }
                    }
                    WsServerMessage::AuthInvalid => {
                        clear_auth_session();
                        last_ws_auth_token.set(None);
                        last_etag.set(None);
                        current_user.set(None);
                    }
                    WsServerMessage::Pong { .. } | WsServerMessage::Unknown => {}
                }
            });

            let onclose = Closure::<dyn FnMut(web_sys::CloseEvent)>::new(move |_| {
                ws_connected.set(false);
                let fails = (ws_failures.get_untracked() + 1).min(5);
                ws_failures.set(fails);
                let backoff_ms = (1500.0 * (1u32 << (fails - 1)) as f64).min(15_000.0);
                next_ws_retry_ms.set(js_sys::Date::now() + backoff_ms);
                if sync_status.get_untracked() != SyncStatus::FallbackHttp {
                    sync_status.set(SyncStatus::Offline);
                }
            });

            socket.set_onopen(Some(onopen.as_ref().unchecked_ref()));
            socket.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
            socket.set_onclose(Some(onclose.as_ref().unchecked_ref()));

            *ws_holder.borrow_mut() = Some(ActiveWsConnection {
                socket,
                _onopen: onopen,
                _onmessage: onmessage,
                _onclose: onclose,
            });
        })
    };

    // Connect on mount & seamlessly re-authenticate over the open WebSocket on login/logout
    {
        let connect_on_auth = Rc::clone(&connect_ws);
        let ws_holder_auth = Rc::clone(&ws_holder);
        create_effect(move |_| {
            let new_token = current_user.get().map(|u| u.token);
            let sent_over_live_ws = {
                let guard = ws_holder_auth.borrow();
                if let Some(ref conn) = *guard {
                    if conn.socket.ready_state() == web_sys::WebSocket::OPEN {
                        if last_ws_auth_token.get_untracked() != new_token {
                            let auth_msg = serde_json::json!({
                                "type": "auth",
                                "token": new_token.clone().unwrap_or_default(),
                            });
                            if conn.socket.send_with_str(&auth_msg.to_string()).is_ok() {
                                last_ws_auth_token.set(new_token.clone());
                            }
                        }
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            };

            if !sent_over_live_ws {
                next_ws_retry_ms.set(0.0);
                connect_on_auth(false);
            }
        });
    }

    // WebSocket Supervisor (25s Keep-Alive Ping + Auto-Reconnect + Safety-Net HTTP Fallback + Network Listeners)
    {
        let connect_sup = Rc::clone(&connect_ws);
        let fallback_sup = Rc::clone(&run_http_fallback_sync);
        let ws_holder_sup = Rc::clone(&ws_holder);

        create_effect(move |already_bound: Option<()>| {
            if already_bound.is_some() {
                return;
            }

            let connect_tick = Rc::clone(&connect_sup);
            let fallback_tick = Rc::clone(&fallback_sup);
            let ws_holder_tick = Rc::clone(&ws_holder_sup);

            let _ = set_interval_with_handle(
                move || {
                    let now = js_sys::Date::now();
                    if ws_connected.get_untracked() {
                        // Send lightweight WebSocket heartbeat every 25s so reverse proxies never drop idle sockets
                        if now - last_ping_ms.get_untracked() >= 25_000.0 {
                            let guard = ws_holder_tick.borrow();
                            if let Some(ref conn) = *guard {
                                if conn.socket.ready_state() == web_sys::WebSocket::OPEN {
                                    let _ = conn.socket.send_with_str("{\"type\":\"ping\"}");
                                    last_ping_ms.set(now);
                                }
                            }
                        }
                        return;
                    }

                    // WebSocket is currently disconnected: check visibility & online state
                    let is_hidden = web_sys::window()
                        .and_then(|w| w.document())
                        .map(|d| d.hidden())
                        .unwrap_or(false);
                    if is_hidden {
                        return;
                    }

                    if now >= next_ws_retry_ms.get_untracked() {
                        connect_tick(false);
                    }

                    // If WebSocket failed multiple times (e.g. corporate proxy blocking Upgrade), keep map live via HTTP ETag
                    if ws_failures.get_untracked() >= 2 && now - last_http_fallback_ms.get_untracked() >= 10_000.0 {
                        fallback_tick();
                    }
                },
                std::time::Duration::from_secs(3),
            );

            if let Some(win) = web_sys::window() {
                if let Some(doc) = win.document() {
                    let connect_vis = Rc::clone(&connect_sup);
                    let ws_holder_vis = Rc::clone(&ws_holder_sup);
                    let vis_cb = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                        let visible = web_sys::window()
                            .and_then(|w| w.document())
                            .map(|d| !d.hidden())
                            .unwrap_or(true);
                        if visible {
                            if ws_connected.get_untracked() {
                                let guard = ws_holder_vis.borrow();
                                if let Some(ref conn) = *guard {
                                    if conn.socket.ready_state() == web_sys::WebSocket::OPEN {
                                        let _ = conn.socket.send_with_str("{\"type\":\"ping\"}");
                                        last_ping_ms.set(js_sys::Date::now());
                                        return;
                                    }
                                }
                            }
                            next_ws_retry_ms.set(0.0);
                            connect_vis(false);
                        }
                    });
                    let _ = doc.add_event_listener_with_callback(
                        "visibilitychange",
                        vis_cb.as_ref().unchecked_ref(),
                    );
                    vis_cb.forget();
                }

                let connect_online = Rc::clone(&connect_sup);
                let online_cb = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                    ws_failures.set(0);
                    next_ws_retry_ms.set(0.0);
                    connect_online(true);
                });
                let _ = win.add_event_listener_with_callback(
                    "online",
                    online_cb.as_ref().unchecked_ref(),
                );
                online_cb.forget();

                let offline_cb = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                    ws_connected.set(false);
                    sync_status.set(SyncStatus::Offline);
                });
                let _ = win.add_event_listener_with_callback(
                    "offline",
                    offline_cb.as_ref().unchecked_ref(),
                );
                offline_cb.forget();
            }
        });
    }

    let on_manual_sync = {
        let connect_manual = Rc::clone(&connect_ws);
        let ws_holder_manual = Rc::clone(&ws_holder);
        Callback::new(move |_| {
            let requested_over_ws = {
                let guard = ws_holder_manual.borrow();
                if let Some(ref conn) = *guard {
                    if conn.socket.ready_state() == web_sys::WebSocket::OPEN {
                        conn.socket
                            .send_with_str("{\"type\":\"request_snapshot\"}")
                            .is_ok()
                    } else {
                        false
                    }
                } else {
                    false
                }
            };
            if !requested_over_ws {
                ws_failures.set(0);
                next_ws_retry_ms.set(0.0);
                connect_manual(true);
            }
        })
    };

    // Reactive memo for filtered alerts
    let filtered_alerts = create_memo(move |_| {
        let all = alerts.get();
        let cats = selected_categories.get();
        let sevs = selected_severities.get();
        let dist = selected_district.get();
        let query = search_query.get().to_lowercase();
        let stat = status_filter.get();
        let days = days_filter.get();

        all.into_iter()
            .filter(|a| cats.contains(&a.category))
            .filter(|a| sevs.contains(&a.severity))
            .filter(|a| dist.map_or(true, |d| a.district == d))
            .filter(|a| {
                if stat == "pending" {
                    a.status == "pending"
                } else if stat == "verified" {
                    a.status == "verified"
                } else if stat == "solved" {
                    a.status == "solved"
                } else if stat == "dismissed" {
                    a.status == "dismissed"
                } else {
                    true
                }
            })
            .filter(|a| {
                if days == 0 {
                    true
                } else {
                    let parsed_ms = js_sys::Date::parse(&a.created_at);
                    if !parsed_ms.is_nan() {
                        let now_ms = js_sys::Date::now();
                        let diff_ms = (now_ms - parsed_ms).max(0.0);
                        let max_ms = (days as f64) * 86_400_000.0;
                        diff_ms <= max_ms
                    } else {
                        true
                    }
                }
            })
            .filter(|a| {
                if query.is_empty() {
                    true
                } else {
                    a.title.to_lowercase().contains(&query)
                        || a.description.to_lowercase().contains(&query)
                        || a.address.to_lowercase().contains(&query)
                }
            })
            .collect::<Vec<EcoAlert>>()
    });

    view! {
        <div class="w-screen h-screen flex flex-row overflow-hidden bg-slate-50 font-sans antialiased text-slate-900 relative">
            // Floating Real-Time Critical Alert Banner
            <EmergencyBanner
                active_emergency=active_emergency_alert
                inspection_target=inspection_target
                selected_alert_id=selected_alert_id
            />

            // Command Center Sidebar
            <Sidebar
                alerts=alerts
                filtered_alerts=filtered_alerts
                selected_categories=selected_categories
                selected_severities=selected_severities
                selected_district=selected_district
                search_query=search_query
                is_report_mode=is_report_mode
                current_user=current_user
                show_auth_dialog=show_auth_dialog
                show_transparency_portal=show_transparency_portal
                show_authority_dashboard=show_authority_dashboard
                selected_alert_id=selected_alert_id
                show_help_guide=show_help_guide
                status_filter=status_filter
                days_filter=days_filter
                sync_status=sync_status
                on_manual_sync=on_manual_sync
                show_user_profile=show_user_profile
                is_collapsed=is_sidebar_collapsed
            />

            // Map View
            <MapView
                filtered_alerts=filtered_alerts
                selected_alert_id=selected_alert_id
                is_report_mode=is_report_mode
                report_center=report_center
                show_new_report_dialog=show_new_report_dialog
                waste_points=waste_points
                collection_routes=collection_routes
                truck_runs=truck_runs
                show_collector_panel=show_collector_panel
                show_waste_layer=show_waste_layer
                selected_waste_point_id=selected_waste_point_id
                show_waste_list_modal=show_waste_list_modal
                show_transparency_portal=show_transparency_portal
                show_authority_dashboard=show_authority_dashboard
                current_user=current_user
                show_help_guide=show_help_guide
                is_sidebar_collapsed=is_sidebar_collapsed
            />

            // Modo Recolector (solo conductores del grupo `recolectores`)
            <CollectorPanel
                current_user=current_user
                show_panel=show_collector_panel
            />

            // Alert Detail floating card
            <AlertDetail
                alerts=alerts
                selected_alert_id=selected_alert_id
                current_user=current_user
                show_auth_dialog=show_auth_dialog
                inspection_target=inspection_target
            />

            // Waste Point Detail floating card
            <WasteDetail
                waste_points=waste_points
                selected_waste_point_id=selected_waste_point_id
                current_gps_or_center=report_center
                current_user=current_user
                show_auth_dialog=show_auth_dialog
            />

            // Waste List Modal
            <WasteListModal
                waste_points=waste_points
                show_waste_list_modal=show_waste_list_modal
                selected_waste_point_id=selected_waste_point_id
                show_waste_layer=show_waste_layer
                current_gps_or_center=report_center
            />

            // Transparency Portal with Before/After comparison
            <TransparencyPortal
                alerts=alerts
                show_transparency_portal=show_transparency_portal
                inspection_target=inspection_target
            />

            // Authority Dashboard with SLA & Action Handlers
            <AuthorityDashboard
                alerts=alerts
                waste_points=waste_points
                show_dashboard=show_authority_dashboard
                current_user=current_user
                show_auth_dialog=show_auth_dialog
                inspection_target=inspection_target
                selected_alert_id=selected_alert_id
                selected_waste_point_id=selected_waste_point_id
                show_waste_layer=show_waste_layer
                truck_runs=truck_runs
            />

            // Report Incident Dialog
            <ReportDialog
                alerts=alerts
                show_new_report_dialog=show_new_report_dialog
                is_report_mode=is_report_mode
                report_center=report_center
                current_user=current_user
                show_auth_dialog=show_auth_dialog
            />

            // Authentication Dialog
            <AuthDialog
                show_dialog=show_auth_dialog
                current_user=current_user
            />

            // Official Municipal Inspection Report & PDF Print Modal
            <InspectionReportModal
                report_target=inspection_target
            />

            // Citizen Help Guide Modal
            <HelpGuideModal
                show_modal=show_help_guide
            />

            // Citizen Profile & Eco-Points Ranking Modal
            <UserProfileModal
                show_modal=show_user_profile
                current_user=current_user
            />

            // Mobile Bottom Navigation Bar (Modern Compact Thumb Navigation)
            <nav class="md:hidden fixed bottom-0 inset-x-0 z-30 bg-white/95 backdrop-blur-xl border-t border-slate-200/90 shadow-[0_-4px_20px_rgba(0,0,0,0.06)] px-2 py-0.5 flex items-center justify-around safe-area-bottom select-none">
                // 1. Explorar Mapa
                <button
                    class="flex flex-col items-center justify-center py-0.5 px-2.5 rounded-xl text-slate-600 hover:text-emerald-700 active:scale-90 transition cursor-pointer"
                    on:click=move |_| {
                        selected_alert_id.set(None);
                        selected_waste_point_id.set(None);
                        show_authority_dashboard.set(false);
                        show_transparency_portal.set(false);
                        is_sidebar_collapsed.set(true);
                    }
                >
                    <i class="fa-solid fa-map-location-dot text-sm text-emerald-600"></i>
                    <span class="text-[9px] font-bold mt-0.5">"Mapa"</span>
                </button>

                // 2. Acopio & Reciclaje
                <button
                    class="flex flex-col items-center justify-center py-0.5 px-2.5 rounded-xl text-slate-600 hover:text-emerald-700 active:scale-90 transition cursor-pointer"
                    on:click=move |_| {
                        show_waste_layer.set(true);
                        show_waste_list_modal.set(true);
                    }
                >
                    <i class="fa-solid fa-recycle text-sm text-teal-600"></i>
                    <span class="text-[9px] font-bold mt-0.5">"Acopio"</span>
                </button>

                // 3. Botón Central Hero Elevado: DENUNCIAR / CONFIRMAR
                <button
                    class=move || {
                        if is_report_mode.get() {
                            "relative -top-2 w-12 h-12 rounded-full bg-purple-600 text-white flex flex-col items-center justify-center shadow-[0_4px_14px_rgba(147,51,234,0.4)] border-2 border-white active:scale-95 transition-transform cursor-pointer"
                        } else {
                            "relative -top-2 w-12 h-12 rounded-full bg-emerald-600 text-white flex flex-col items-center justify-center shadow-[0_4px_14px_rgba(5,150,105,0.35)] border-2 border-white active:scale-95 transition-transform cursor-pointer"
                        }
                    }
                    on:click=move |_| {
                        if current_user.get().is_none() {
                            show_auth_dialog.set(true);
                        } else if is_report_mode.get() {
                            // Si ya está ubicando en el mapa interactivo, confirma y abre el modal
                            show_mobile_report_options.set(false);
                            show_new_report_dialog.set(true);
                        } else {
                            // En móvil: muestra las opciones claras de ubicación (Mapa o GPS)
                            show_mobile_report_options.set(true);
                        }
                    }
                    title="Registrar denuncia ambiental con foto y GPS"
                >
                    {move || if is_report_mode.get() {
                        view! {
                            <i class="fa-solid fa-check text-sm"></i>
                            <span class="text-[7.5px] font-extrabold uppercase tracking-tight -mt-0.5">"Listo"</span>
                        }.into_view()
                    } else {
                        view! {
                            <i class="fa-solid fa-bullhorn text-sm"></i>
                            <span class="text-[7.5px] font-extrabold uppercase tracking-tight -mt-0.5">"Denunciar"</span>
                        }.into_view()
                    }}
                </button>

                // 4. Filtros & Estadísticas
                <button
                    class="flex flex-col items-center justify-center py-0.5 px-2.5 rounded-xl text-slate-600 hover:text-emerald-700 active:scale-90 transition cursor-pointer"
                    on:click=move |_| {
                        is_sidebar_collapsed.update(|v| *v = !*v);
                    }
                >
                    <i class="fa-solid fa-sliders text-sm text-sky-600"></i>
                    <span class="text-[9px] font-bold mt-0.5">"Filtros"</span>
                </button>

                // 5. Mis Eco-Puntos / Perfil
                <button
                    class="flex flex-col items-center justify-center py-0.5 px-2.5 rounded-xl text-slate-600 hover:text-emerald-700 active:scale-90 transition cursor-pointer"
                    on:click=move |_| {
                        if current_user.get().is_none() {
                            show_auth_dialog.set(true);
                        } else {
                            show_user_profile.set(true);
                        }
                    }
                >
                    <i class="fa-solid fa-trophy text-sm text-amber-500"></i>
                    <span class="text-[9px] font-bold mt-0.5">"Puntos"</span>
                </button>
            </nav>

            // Mobile Interactive Report Options Drawer (Action Sheet)
            {move || show_mobile_report_options.get().then(|| {
                view! {
                    <div class="fixed inset-0 z-50 md:hidden flex flex-col justify-end">
                        // Backdrop overlay
                        <div
                            class="fixed inset-0 bg-slate-900/60 backdrop-blur-xs transition-opacity animate-in fade-in duration-200"
                            on:click=move |_| show_mobile_report_options.set(false)
                        ></div>

                        // Drawer Sheet
                        <div class="relative z-10 bg-white rounded-t-3xl p-5 shadow-[0_-12px_40px_rgba(0,0,0,0.22)] border-t border-slate-200/90 space-y-4 animate-in slide-in-from-bottom duration-250 select-none">
                            // Drag Handle
                            <div class="w-12 h-1.5 bg-slate-200 rounded-full mx-auto -mt-1 mb-1"></div>

                            // Header
                            <div class="flex items-center justify-between">
                                <div class="flex items-center space-x-2.5">
                                    <div class="w-9 h-9 rounded-xl bg-purple-50 text-purple-700 border border-purple-200 flex items-center justify-center font-bold">
                                        <i class="fa-solid fa-bullhorn text-sm"></i>
                                    </div>
                                    <div>
                                        <h3 class="text-sm font-extrabold text-slate-900 leading-tight">
                                            "Registrar Denuncia Ambiental"
                                        </h3>
                                        <p class="text-[11px] text-slate-500 font-medium">
                                            "¿Cómo deseas fijar la ubicación del foco?"
                                        </p>
                                    </div>
                                </div>
                                <button
                                    class="w-7 h-7 rounded-full bg-slate-100 hover:bg-slate-200 text-slate-500 hover:text-slate-800 flex items-center justify-center transition cursor-pointer"
                                    on:click=move |_| show_mobile_report_options.set(false)
                                >
                                    <i class="fa-solid fa-xmark text-xs"></i>
                                </button>
                            </div>

                            // Option Cards Container (Flat Apple Style)
                            <div class="space-y-2.5 pt-1">
                                // Option 1: Posicionar en el Mapa Interactivo
                                <button
                                    type="button"
                                    class="w-full p-3.5 rounded-2xl bg-slate-50 hover:bg-slate-100 active:bg-slate-200/80 border border-slate-200 flex items-center space-x-3 text-left transition cursor-pointer shadow-2xs group"
                                    on:click=move |_| {
                                        show_mobile_report_options.set(false);
                                        is_sidebar_collapsed.set(true);
                                        is_report_mode.set(true);
                                        let _ = js_sys::eval(r#"
                                            if (window.ecoMap && window.ecoMap.map) {
                                                window.ecoMap.map.invalidateSize({ animate: false });
                                            }
                                        "#);
                                    }
                                >
                                    <div class="w-10 h-10 rounded-xl bg-purple-600 text-white flex items-center justify-center shadow-xs shrink-0">
                                        <i class="fa-solid fa-map-pin text-sm"></i>
                                    </div>
                                    <div class="flex-1 min-w-0">
                                        <div class="flex items-center space-x-1.5">
                                            <span class="text-xs font-bold text-slate-900">"Ubicar en el Mapa"</span>
                                            <span class="px-1.5 py-0.2 rounded-md bg-purple-100 text-[9px] font-bold text-purple-700">"Recomendado"</span>
                                        </div>
                                        <p class="text-[11px] text-slate-500 leading-snug mt-0.5 font-normal">
                                            "Mueve el mapa y posiciona el marcador en el lugar exacto del incidente."
                                        </p>
                                    </div>
                                    <i class="fa-solid fa-chevron-right text-slate-400 group-hover:text-slate-700 text-xs transition-transform group-hover:translate-x-0.5"></i>
                                </button>

                                // Option 2: Usar mi Ubicación GPS
                                <button
                                    type="button"
                                    class="w-full p-3.5 rounded-2xl bg-slate-50 hover:bg-slate-100 active:bg-slate-200/80 border border-slate-200 flex items-center space-x-3 text-left transition cursor-pointer shadow-2xs group"
                                    on:click=move |_| {
                                        show_mobile_report_options.set(false);
                                        is_sidebar_collapsed.set(true);
                                        spawn_local(async move {
                                            if let Ok((lat, lng)) = crate::geo::get_browser_gps_position().await {
                                                report_center.set((lat, lng));
                                                let _ = js_sys::eval(&format!(
                                                    "if (window.ecoMap && window.ecoMap.setUserLocation) window.ecoMap.setUserLocation({}, {});",
                                                    lat, lng
                                                ));
                                            }
                                            show_new_report_dialog.set(true);
                                        });
                                    }
                                >
                                    <div class="w-10 h-10 rounded-xl bg-slate-900 text-white flex items-center justify-center shadow-xs shrink-0">
                                        <i class="fa-solid fa-location-crosshairs text-sm"></i>
                                    </div>
                                    <div class="flex-1 min-w-0">
                                        <span class="text-xs font-bold text-slate-900 block">"Usar Mi Ubicación GPS"</span>
                                        <p class="text-[11px] text-slate-500 leading-snug mt-0.5 font-normal">
                                            "Detectar tus coordenadas automáticas y abrir el reporte de inmediato."
                                        </p>
                                    </div>
                                    <i class="fa-solid fa-chevron-right text-slate-400 group-hover:text-slate-700 text-xs transition-transform group-hover:translate-x-0.5"></i>
                                </button>
                            </div>

                            // Cancel Button
                            <button
                                type="button"
                                class="w-full py-3 rounded-xl bg-slate-100 hover:bg-slate-200 text-slate-700 font-semibold text-xs transition cursor-pointer"
                                on:click=move |_| show_mobile_report_options.set(false)
                            >
                                "Cancelar"
                            </button>
                        </div>
                    </div>
                }
            })}
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
