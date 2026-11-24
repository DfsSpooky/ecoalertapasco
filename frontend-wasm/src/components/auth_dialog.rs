use leptos::*;
use crate::api::{
    google_auth_config_url, google_auth_url, login_url, profile_url, register_url,
};
use crate::models::{
    AuthUser, GoogleAuthConfig, GoogleAuthPayload, LoginPayload, RegisterPayload,
};
use crate::storage::{
    clear_remembered_google_user, load_remembered_google_user, save_auth_session,
    save_remembered_google_user,
};

fn is_in_app_webview() -> bool {
    js_sys::eval("Boolean(window.ecoGoogleAuth && window.ecoGoogleAuth.isInAppBrowser())")
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn render_google_button_soon(delay_ms: u32) {
    let _ = js_sys::eval(&format!(
        "setTimeout(function() {{ if (window.ecoGoogleAuth) window.ecoGoogleAuth.renderButton('google-gis-btn-container'); }}, {})",
        delay_ms
    ));
}

#[component]
pub fn AuthDialog(
    show_dialog: RwSignal<bool>,
    current_user: RwSignal<Option<AuthUser>>,
) -> impl IntoView {
    // Tabs: "usuarios" | "autoridades"
    let active_tab = create_rw_signal("usuarios".to_string());

    // Form inputs for municipal authorities
    let login_username = create_rw_signal("".to_string());
    let login_password = create_rw_signal("".to_string());

    // Fallback ciudadano con correo/usuario y registro Turnstile
    let show_email_fallback = create_rw_signal(false);
    let email_auth_mode = create_rw_signal("login".to_string()); // "login" | "register"
    let citizen_identifier = create_rw_signal("".to_string());
    let citizen_password = create_rw_signal("".to_string());
    let reg_username = create_rw_signal("".to_string());
    let reg_email = create_rw_signal("".to_string());
    let reg_password = create_rw_signal("".to_string());
    let turnstile_token = create_rw_signal("".to_string());

    let is_submitting = create_rw_signal(false);
    let error_message = create_rw_signal(None::<String>);
    let info_message = create_rw_signal(None::<String>);
    let in_app_browser = create_rw_signal(false);

    // Google Auth State
    let google_config = create_rw_signal(None::<GoogleAuthConfig>);

    // Detected / Remembered Google Account
    let remembered_user = create_rw_signal(load_remembered_google_user());
    let switch_to_other_account = create_rw_signal(false);

    let render_turnstile_if_needed = move || {
        let is_open = show_dialog.get_untracked();
        let is_reg = active_tab.get_untracked() == "usuarios"
            && show_email_fallback.get_untracked()
            && email_auth_mode.get_untracked() == "register";
        if !is_open || !is_reg {
            return;
        }
        let cfg_opt = google_config.get_untracked();
        let enabled = cfg_opt
            .as_ref()
            .and_then(|c| c.turnstile_enabled)
            .unwrap_or(true);
        if !enabled {
            return;
        }
        let sitekey = cfg_opt
            .and_then(|c| c.turnstile_sitekey)
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "1x00000000000000000000AA".to_string());
        if let Ok(key_json) = serde_json::to_string(&sitekey) {
            let _ = js_sys::eval(&format!(
                "setTimeout(function() {{ if (window.ecoTurnstile) window.ecoTurnstile.render('eco-turnstile-register-container', {}); }}, 80)",
                key_json
            ));
        }
    };

    // Fetch Google configuration when dialog is opened & sync modal flag
    create_effect(move |_| {
        let is_open = show_dialog.get();
        let _ = js_sys::eval(&format!("window.isAuthModalOpen = {};", is_open));
        if is_open {
            error_message.set(None);
            info_message.set(None);
            remembered_user.set(load_remembered_google_user());
            switch_to_other_account.set(false);
            let webview = is_in_app_webview();
            in_app_browser.set(webview);
            if webview {
                show_email_fallback.set(true);
            }

            // Fetch Google configuration once if not cached
            if google_config.get_untracked().is_none() {
                spawn_local(async move {
                    let req = gloo_net::http::Request::get(&google_auth_config_url()).send().await;
                    if let Ok(resp) = req {
                        if resp.ok() {
                            if let Ok(cfg) = resp.json::<GoogleAuthConfig>().await {
                                let cid = cfg.client_id.clone();
                                let configured = cfg.is_configured;
                                google_config.set(Some(cfg));
                                if configured && !cid.trim().is_empty() {
                                    let _ = js_sys::eval(&format!("window.ecoGoogleAuth.init('{}')", cid));
                                    render_google_button_soon(100);
                                } else {
                                    show_email_fallback.set(true);
                                }
                                render_turnstile_if_needed();
                            }
                        }
                    }
                });
            } else {
                if let Some(cfg) = google_config.get_untracked() {
                    if cfg.is_configured {
                        render_google_button_soon(80);
                    } else {
                        show_email_fallback.set(true);
                    }
                }
                render_turnstile_if_needed();
            }
        } else {
            let _ = js_sys::eval("if (window.ecoTurnstile) window.ecoTurnstile.remove();");
        }
    });

    let close_dialog = move |_| {
        let _ = js_sys::eval("window.isAuthModalOpen = false;");
        let _ = js_sys::eval("if (window.ecoTurnstile) window.ecoTurnstile.remove();");
        error_message.set(None);
        info_message.set(None);
        show_dialog.set(false);
    };

    let switch_tab = move |tab: &'static str| {
        error_message.set(None);
        info_message.set(None);
        active_tab.set(tab.to_string());
        if tab == "usuarios" {
            render_google_button_soon(80);
            render_turnstile_if_needed();
        }
    };

    // Handle Google Login & Registration Execution
    let perform_google_login = {
        move |credential: String| {
            is_submitting.set(true);
            error_message.set(None);
            info_message.set(None);

            spawn_local(async move {
                let payload = GoogleAuthPayload {
                    credential: &credential,
                };

                let req_res = gloo_net::http::Request::post(&google_auth_url())
                    .header("Content-Type", "application/json")
                    .json(&payload);

                match req_res {
                    Ok(req) => match req.send().await {
                        Ok(resp) => {
                            is_submitting.set(false);
                            if resp.ok() {
                                if let Ok(user) = resp.json::<AuthUser>().await {
                                    save_auth_session(&user);
                                    remembered_user.set(Some(user.clone()));
                                    current_user.set(Some(user));
                                    show_dialog.set(false);
                                } else {
                                    error_message.set(Some("Respuesta inesperada al autenticar con Google.".to_string()));
                                }
                            } else if let Ok(err_val) = resp.json::<serde_json::Value>().await {
                                if let Some(err_str) = err_val.get("error").and_then(|v| v.as_str()) {
                                    error_message.set(Some(err_str.to_string()));
                                } else {
                                    error_message.set(Some("No se pudo completar el acceso con Google.".to_string()));
                                }
                            } else {
                                error_message.set(Some("Token de Google rechazado por el servidor.".to_string()));
                            }
                        }
                        Err(e) => {
                            is_submitting.set(false);
                            error_message.set(Some(format!("Error de conexión con Google: {}", e)));
                        }
                    },
                    Err(e) => {
                        is_submitting.set(false);
                        error_message.set(Some(format!("Error serializando solicitud Google: {}", e)));
                    }
                }
            });
        }
    };

    // Shared Login handler (both Authority tab and Citizen email/password fallback)
    let execute_password_login = move |identifier: String, password: String, clear_on_ok: bool| {
        if identifier.trim().is_empty() || password.trim().is_empty() {
            error_message.set(Some("Por favor ingresa tu usuario/correo y contraseña.".to_string()));
            return;
        }

        is_submitting.set(true);
        error_message.set(None);
        info_message.set(None);

        spawn_local(async move {
            let payload = LoginPayload {
                username: identifier.trim(),
                password: &password,
            };

            let req_res = gloo_net::http::Request::post(&login_url())
                .header("Content-Type", "application/json")
                .json(&payload);

            match req_res {
                Ok(req) => match req.send().await {
                    Ok(resp) => {
                        is_submitting.set(false);
                        if resp.ok() {
                            if let Ok(user) = resp.json::<AuthUser>().await {
                                save_auth_session(&user);
                                current_user.set(Some(user));
                                if clear_on_ok {
                                    login_username.set("".to_string());
                                    login_password.set("".to_string());
                                    citizen_identifier.set("".to_string());
                                    citizen_password.set("".to_string());
                                }
                                show_dialog.set(false);
                            } else {
                                error_message.set(Some("Respuesta inesperada del servidor.".to_string()));
                            }
                        } else {
                            let status = resp.status();
                            if status == 429 {
                                error_message.set(Some("Demasiados intentos fallidos. Espera un momento antes de reintentar.".to_string()));
                            } else if let Ok(err_val) = resp.json::<serde_json::Value>().await {
                                if let Some(err_str) = err_val.get("error").and_then(|v| v.as_str()) {
                                    error_message.set(Some(err_str.to_string()));
                                } else if let Some(detail) = err_val.get("detail").and_then(|v| v.as_str()) {
                                    error_message.set(Some(detail.to_string()));
                                } else {
                                    error_message.set(Some(format!("Error de autenticación ({})", status)));
                                }
                            } else {
                                error_message.set(Some("Credenciales inválidas. Verifica usuario y contraseña.".to_string()));
                            }
                        }
                    }
                    Err(e) => {
                        is_submitting.set(false);
                        error_message.set(Some(format!("No se pudo conectar al servidor: {}", e)));
                    }
                },
                Err(e) => {
                    is_submitting.set(false);
                    error_message.set(Some(format!("Error serializando petición: {}", e)));
                }
            }
        });
    };

    let perform_authority_login = move || {
        execute_password_login(login_username.get(), login_password.get(), true);
    };

    let perform_citizen_login = move || {
        execute_password_login(citizen_identifier.get(), citizen_password.get(), true);
    };

    let perform_citizen_register = move || {
        let u = reg_username.get();
        let e = reg_email.get();
        let p = reg_password.get();

        if u.trim().is_empty() || e.trim().is_empty() || p.is_empty() {
            error_message.set(Some("Completa usuario, correo y contraseña para registrarte.".to_string()));
            return;
        }
        if p.len() < 8 {
            error_message.set(Some("La contraseña debe tener al menos 8 caracteres.".to_string()));
            return;
        }

        let turnstile_enabled = google_config
            .get_untracked()
            .and_then(|c| c.turnstile_enabled)
            .unwrap_or(true);

        let mut tok = turnstile_token.get_untracked();
        if tok.trim().is_empty() {
            if let Ok(js_tok) = js_sys::eval("window.ecoTurnstile ? window.ecoTurnstile.getToken() : ''") {
                if let Some(s) = js_tok.as_string() {
                    tok = s;
                }
            }
        }
        if turnstile_enabled && tok.trim().is_empty() {
            error_message.set(Some("Completa la verificación anti-bot antes de crear tu cuenta.".to_string()));
            return;
        }

        is_submitting.set(true);
        error_message.set(None);
        info_message.set(None);

        spawn_local(async move {
            let tok_opt = if tok.trim().is_empty() { None } else { Some(tok.trim()) };
            let payload = RegisterPayload {
                username: u.trim(),
                email: e.trim(),
                password: &p,
                turnstile_token: tok_opt,
            };

            let req_res = gloo_net::http::Request::post(&register_url())
                .header("Content-Type", "application/json")
                .json(&payload);

            match req_res {
                Ok(req) => match req.send().await {
                    Ok(resp) => {
                        is_submitting.set(false);
                        if resp.ok() {
                            if let Ok(user) = resp.json::<AuthUser>().await {
                                save_auth_session(&user);
                                current_user.set(Some(user));
                                reg_username.set("".to_string());
                                reg_email.set("".to_string());
                                reg_password.set("".to_string());
                                turnstile_token.set("".to_string());
                                let _ = js_sys::eval("if (window.ecoTurnstile) window.ecoTurnstile.remove();");
                                show_dialog.set(false);
                            } else {
                                error_message.set(Some("Respuesta inesperada al registrar usuario.".to_string()));
                            }
                        } else {
                            let _ = js_sys::eval("if (window.ecoTurnstile) window.ecoTurnstile.reset();");
                            turnstile_token.set("".to_string());
                            if resp.status() == 429 {
                                error_message.set(Some("Demasiados registros recientes desde esta red. Intenta más tarde.".to_string()));
                            } else if let Ok(err_val) = resp.json::<serde_json::Value>().await {
                                if let Some(err_str) = err_val.get("error").and_then(|v| v.as_str()) {
                                    error_message.set(Some(err_str.to_string()));
                                } else {
                                    error_message.set(Some(format!("Error al registrar cuenta ({})", resp.status())));
                                }
                            } else {
                                error_message.set(Some("No se pudo completar el registro.".to_string()));
                            }
                        }
                    }
                    Err(e) => {
                        is_submitting.set(false);
                        error_message.set(Some(format!("Error de conexión al registrar: {}", e)));
                    }
                },
                Err(e) => {
                    is_submitting.set(false);
                    error_message.set(Some(format!("Error serializando registro: {}", e)));
                }
            }
        });
    };

    let login_with_detected = move |_| {
        if let Some(mut user) = remembered_user.get_untracked() {
            if user.token.trim().is_empty() {
                switch_to_other_account.set(true);
                info_message.set(Some("Por seguridad, confirma tu cuenta haciendo clic en el botón de Google.".to_string()));
                render_google_button_soon(60);
                return;
            }

            is_submitting.set(true);
            error_message.set(None);
            info_message.set(None);

            spawn_local(async move {
                let req = gloo_net::http::Request::get(&profile_url())
                    .header("Authorization", &format!("Token {}", user.token))
                    .send()
                    .await;

                if let Ok(resp) = req {
                    if resp.ok() {
                        save_auth_session(&user);
                        current_user.set(Some(user));
                        is_submitting.set(false);
                        show_dialog.set(false);
                        return;
                    }
                }

                // Token expirado o revocado: limpiar token muerto y mostrar botón oficial de Google
                user.token = String::new();
                save_remembered_google_user(&user);
                remembered_user.set(Some(user));
                switch_to_other_account.set(true);
                info_message.set(Some("Tu sesión anterior expiró. Confirma tu identidad con el botón oficial de Google.".to_string()));
                is_submitting.set(false);
                render_google_button_soon(60);
            });
        }
    };

    let handle_use_other_account = move |_| {
        let _ = js_sys::eval("if (window.ecoGoogleAuth) window.ecoGoogleAuth.disableAutoSelect();");
        info_message.set(None);
        switch_to_other_account.set(true);
        render_google_button_soon(60);
    };

    let handle_forget_account = move |_| {
        let _ = js_sys::eval("if (window.ecoGoogleAuth) window.ecoGoogleAuth.disableAutoSelect();");
        clear_remembered_google_user();
        remembered_user.set(None);
        info_message.set(None);
        switch_to_other_account.set(true);
        render_google_button_soon(60);
    };

    view! {
        {move || show_dialog.get().then(|| {
            let is_usuarios = move || active_tab.get() == "usuarios";
            let google_login_fn = perform_google_login.clone();

            view! {
                <div class="fixed inset-0 bg-slate-900/60 backdrop-blur-sm flex items-end sm:items-center justify-center p-0 sm:p-4 z-50 animate-in fade-in duration-150">
                    <div class="bg-white w-full max-w-md rounded-t-3xl sm:rounded-2xl border border-slate-200/90 shadow-[0_24px_64px_rgba(0,0,0,0.22)] overflow-hidden max-h-[92vh] flex flex-col">
                        // Hidden buttons for GIS & Turnstile dispatch callbacks
                        {
                            let google_dispatch = google_login_fn.clone();
                            view! {
                                <button
                                    id="eco-hidden-google-dispatch-btn"
                                    class="hidden"
                                    type="button"
                                    on:click=move |_| {
                                        if let Ok(val) = js_sys::eval("window.ecoLastGoogleCredential || ''") {
                                            if let Some(cred) = val.as_string() {
                                                if !cred.is_empty() {
                                                    let _ = js_sys::eval("window.ecoLastGoogleCredential = ''");
                                                    google_dispatch(cred);
                                                }
                                            }
                                        }
                                    }
                                ></button>
                            }
                        }
                        <button
                            id="eco-hidden-turnstile-dispatch-btn"
                            class="hidden"
                            type="button"
                            on:click=move |_| {
                                if let Ok(val) = js_sys::eval("window.ecoTurnstile ? window.ecoTurnstile.getToken() : ''") {
                                    if let Some(tok) = val.as_string() {
                                        turnstile_token.set(tok);
                                    }
                                }
                            }
                        ></button>

                        // Mobile Drag Pill
                        <div class="w-12 h-1.5 bg-slate-300 rounded-full mx-auto my-2 sm:hidden shrink-0"></div>

                        // Dialog Header
                        <div class="px-6 pt-5 pb-4 border-b border-slate-100 flex items-start justify-between">
                            <div class="flex items-center space-x-3">
                                <div class="w-10 h-10 rounded-xl bg-emerald-500/10 text-emerald-600 border border-emerald-500/20 flex items-center justify-center font-bold text-lg shadow-xs">
                                    <svg class="w-5 h-5 text-emerald-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                                      <path d="M11 20A7 7 0 0 1 9.8 6.1C15.5 5 17 4.48 19 2c1 2 2 4.18 2 8 0 5.5-4.78 10-10 10Z"/>
                                      <path d="M2 21c0-3 1.85-5.36 5.08-6C9.5 14.52 12 13 13 12"/>
                                    </svg>
                                </div>
                                <div>
                                    <h2 class="text-base font-bold text-slate-900 leading-snug">"Acceso a EcoAlerta"</h2>
                                    <p class="text-[11px] text-slate-500 font-medium">"Ambiental y Seguridad Ciudadana · Pasco"</p>
                                </div>
                            </div>
                            <button
                                class="text-slate-400 hover:text-slate-700 p-1 rounded-lg hover:bg-slate-100 transition cursor-pointer"
                                on:click=close_dialog
                            >
                                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <line x1="18" y1="6" x2="6" y2="18"></line>
                                    <line x1="6" y1="6" x2="18" y2="18"></line>
                                </svg>
                            </button>
                        </div>

                        // Modal Scrollable Content
                        <div class="p-6 overflow-y-auto space-y-4">
                            // Info banner
                            {move || info_message.get().map(|msg| view! {
                                <div class="p-3 bg-sky-50 rounded-xl border border-sky-200 flex items-start space-x-2.5 animate-in fade-in">
                                    <i class="fa-solid fa-circle-info text-sky-600 mt-0.5 text-xs shrink-0"></i>
                                    <p class="text-[11px] text-sky-800 leading-snug font-medium">{msg}</p>
                                </div>
                            })}

                            // Error banner
                            {move || error_message.get().map(|msg| view! {
                                <div class="p-3 bg-red-50 rounded-xl border border-red-200 flex items-start space-x-2.5 animate-in fade-in">
                                    <svg class="w-4 h-4 text-red-600 mt-0.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                        <circle cx="12" cy="12" r="10"/>
                                        <line x1="12" y1="8" x2="12" y2="12"/>
                                        <line x1="12" y1="16" x2="12.01" y2="16"/>
                                    </svg>
                                    <p class="text-[11px] text-red-700 leading-snug font-medium">{msg}</p>
                                </div>
                            })}

                            // ==================== TABS: USUARIOS / AUTORIDADES ====================
                            <div class="bg-slate-100 p-1 rounded-xl flex space-x-1 border border-slate-200/70">
                                <button
                                    type="button"
                                    class=move || {
                                        if is_usuarios() {
                                            "flex-1 py-2 rounded-lg text-xs font-bold bg-white text-emerald-800 shadow-xs border border-slate-200/80 transition flex items-center justify-center space-x-1.5 cursor-pointer"
                                        } else {
                                            "flex-1 py-2 rounded-lg text-xs font-semibold text-slate-500 hover:text-slate-800 transition flex items-center justify-center space-x-1.5 cursor-pointer"
                                        }
                                    }
                                    on:click=move |_| switch_tab("usuarios")
                                >
                                    <svg class="w-3.5 h-3.5 text-emerald-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                        <path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/>
                                        <circle cx="12" cy="7" r="4"/>
                                    </svg>
                                    <span>"Ciudadanos"</span>
                                </button>
                                <button
                                    type="button"
                                    class=move || {
                                        if !is_usuarios() {
                                            "flex-1 py-2 rounded-lg text-xs font-bold bg-white text-slate-900 shadow-xs border border-slate-200/80 transition flex items-center justify-center space-x-1.5 cursor-pointer"
                                        } else {
                                            "flex-1 py-2 rounded-lg text-xs font-semibold text-slate-500 hover:text-slate-800 transition flex items-center justify-center space-x-1.5 cursor-pointer"
                                        }
                                    }
                                    on:click=move |_| switch_tab("autoridades")
                                >
                                    <svg class="w-3.5 h-3.5 text-slate-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                        <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
                                    </svg>
                                    <span>"Autoridades"</span>
                                </button>
                            </div>

                            // ==================== PANEL 1: CIUDADANOS (GOOGLE + FALLBACK) ====================
                            <div class=move || if is_usuarios() { "space-y-4 pt-1 animate-in fade-in duration-150" } else { "hidden" }>
                                {move || in_app_browser.get().then(|| view! {
                                    <div class="p-3 rounded-xl bg-amber-50 border border-amber-200 text-[11px] text-amber-900 leading-relaxed space-y-1">
                                        <p class="font-bold flex items-center space-x-1.5">
                                            <span>"⚠️"</span>
                                            <span>"Navegador interno detectado (WhatsApp / Redes)"</span>
                                        </p>
                                        <p>"Google suele bloquear el inicio de sesión dentro de apps. Puedes abrir este enlace en Chrome/Safari o usar correo y contraseña abajo."</p>
                                    </div>
                                })}

                                {move || {
                                    let rem_opt = remembered_user.get();
                                    let has_remembered_active = rem_opt
                                        .as_ref()
                                        .map(|u| !u.token.trim().is_empty())
                                        .unwrap_or(false)
                                        && !switch_to_other_account.get();

                                    if has_remembered_active {
                                        let rem_user = rem_opt.unwrap();
                                        let first_name = rem_user
                                            .full_name
                                            .as_deref()
                                            .unwrap_or(&rem_user.username)
                                            .split_whitespace()
                                            .next()
                                            .unwrap_or("Usuario")
                                            .to_string();

                                        view! {
                                            <div class="p-4 rounded-2xl bg-gradient-to-br from-emerald-50/90 to-teal-50/50 border-2 border-emerald-300 space-y-3.5 shadow-xs">
                                                <div class="flex items-center space-x-3">
                                                    {if let Some(ref photo) = rem_user.photo_url {
                                                        view! {
                                                            <img src=photo.clone() alt="Foto de perfil" class="w-12 h-12 rounded-full border-2 border-emerald-500 shadow-xs object-cover shrink-0" />
                                                        }.into_view()
                                                    } else {
                                                        view! {
                                                            <div class="w-12 h-12 rounded-full bg-emerald-600 text-white font-bold text-lg flex items-center justify-center shadow-xs shrink-0">
                                                                {first_name.chars().next().unwrap_or('U').to_string()}
                                                            </div>
                                                        }.into_view()
                                                    }}
                                                    <div class="min-w-0 flex-1">
                                                        <span class="inline-flex items-center space-x-1 px-2 py-0.5 rounded-full text-[10px] font-bold bg-emerald-100 text-emerald-800">
                                                            <svg class="w-3 h-3 text-emerald-600" viewBox="0 0 24 24" fill="currentColor">
                                                                <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
                                                            </svg>
                                                            <span>"Cuenta de Google Detectada"</span>
                                                        </span>
                                                        <h4 class="text-xs sm:text-sm font-bold text-slate-900 truncate mt-1">
                                                            {rem_user.full_name.clone().unwrap_or_else(|| rem_user.username.clone())}
                                                        </h4>
                                                        <p class="text-[11px] text-slate-500 truncate">
                                                            {rem_user.email.clone().unwrap_or_default()}
                                                        </p>
                                                    </div>
                                                </div>

                                                <button
                                                    type="button"
                                                    class="w-full py-3 px-4 rounded-xl font-bold text-xs sm:text-sm text-white bg-emerald-600 hover:bg-emerald-700 active:bg-emerald-800 shadow-md shadow-emerald-600/25 flex items-center justify-center space-x-2 transition cursor-pointer disabled:opacity-50"
                                                    disabled=move || is_submitting.get()
                                                    on:click=login_with_detected
                                                >
                                                    {move || if is_submitting.get() {
                                                        view! {
                                                            <span class="inline-block w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></span>
                                                            <span>"Verificando sesión..."</span>
                                                        }.into_view()
                                                    } else {
                                                        view! {
                                                            <span>{format!("Continuar como {}", first_name)}</span>
                                                        }.into_view()
                                                    }}
                                                </button>

                                                <div class="pt-2 border-t border-emerald-100 flex items-center justify-between text-[11px]">
                                                    <button
                                                        type="button"
                                                        class="text-slate-600 hover:text-emerald-700 font-semibold transition cursor-pointer flex items-center space-x-1"
                                                        on:click=handle_use_other_account
                                                    >
                                                        <i class="fa-solid fa-arrows-rotate text-[10px] text-emerald-600"></i>
                                                        <span>"Usar otra cuenta"</span>
                                                    </button>
                                                    <button
                                                        type="button"
                                                        class="text-slate-400 hover:text-rose-600 font-medium transition cursor-pointer"
                                                        on:click=handle_forget_account
                                                    >
                                                        "Olvidar cuenta"
                                                    </button>
                                                </div>
                                            </div>
                                        }.into_view()
                                    } else {
                                        let google_ready = google_config.get().map(|c| c.is_configured).unwrap_or(true);
                                        let has_prev_profile = rem_opt.is_some();

                                        view! {
                                            <div class="p-4 rounded-2xl bg-gradient-to-br from-emerald-50/80 to-teal-50/40 border border-emerald-200/80 space-y-3 shadow-2xs">
                                                <div class="flex items-start space-x-2.5">
                                                    <div class="w-7 h-7 rounded-lg bg-emerald-600/10 text-emerald-700 flex items-center justify-center shrink-0 mt-0.5">
                                                        <svg class="w-4 h-4 text-emerald-700" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
                                                        </svg>
                                                    </div>
                                                    <div>
                                                        <h3 class="text-xs font-bold text-emerald-950">"Acceso Rápido con Google"</h3>
                                                        <p class="text-[11px] text-emerald-800 leading-relaxed mt-0.5">
                                                            "Inicia sesión o regístrate en un clic para reportar alertas ambientales o de seguridad ciudadana."
                                                        </p>
                                                    </div>
                                                </div>

                                                {if google_ready {
                                                    view! {
                                                        <div class="space-y-2">
                                                            <div id="google-gis-btn-container" class="w-full flex justify-center min-h-[44px]"></div>
                                                            {move || is_submitting.get().then(|| view! {
                                                                <p class="text-center text-[11px] text-emerald-700 font-semibold animate-pulse">
                                                                    "Validando credencial con Google..."
                                                                </p>
                                                            })}
                                                        </div>
                                                    }.into_view()
                                                } else {
                                                    view! {
                                                        <div class="p-2.5 rounded-xl bg-white/80 border border-slate-200 text-[11px] text-slate-600 text-center">
                                                            "Inicio con Google desactivado en este entorno. Usa correo y contraseña abajo."
                                                        </div>
                                                    }.into_view()
                                                }}

                                                {move || (has_prev_profile && rem_opt.as_ref().map(|u| !u.token.trim().is_empty()).unwrap_or(false) && switch_to_other_account.get()).then(|| {
                                                    view! {
                                                        <div class="pt-2 text-center border-t border-emerald-100">
                                                            <button
                                                                type="button"
                                                                class="text-[11px] text-emerald-700 hover:underline font-semibold cursor-pointer"
                                                                on:click=move |_| switch_to_other_account.set(false)
                                                            >
                                                                "← Volver a mi cuenta detectada"
                                                            </button>
                                                        </div>
                                                    }
                                                })}
                                            </div>
                                        }.into_view()
                                    }
                                }}

                                // Alternativa con Correo y Contraseña (Login + Registro con Turnstile)
                                <div class="border-t border-slate-100 pt-2">
                                    <button
                                        type="button"
                                        class="w-full py-1.5 text-[11px] font-semibold text-slate-500 hover:text-emerald-700 transition flex items-center justify-center space-x-1.5 cursor-pointer"
                                        on:click=move |_| {
                                            let next = !show_email_fallback.get();
                                            show_email_fallback.set(next);
                                            if next {
                                                render_turnstile_if_needed();
                                            }
                                        }
                                    >
                                        <span>
                                            {move || if show_email_fallback.get() {
                                                "Ocultar acceso con correo y contraseña ▲"
                                            } else {
                                                "¿Prefieres ingresar o registrarte con correo y contraseña? ▼"
                                            }}
                                        </span>
                                    </button>

                                    {move || show_email_fallback.get().then(|| {
                                        let is_reg_mode = move || email_auth_mode.get() == "register";
                                        view! {
                                            <div class="mt-3 p-3.5 rounded-2xl bg-slate-50 border border-slate-200/80 space-y-3 animate-in fade-in duration-150">
                                                <div class="grid grid-cols-2 gap-1 bg-slate-200/70 p-0.5 rounded-xl">
                                                    <button
                                                        type="button"
                                                        class=move || if !is_reg_mode() {
                                                            "py-1.5 rounded-lg text-[11px] font-bold bg-white text-slate-900 shadow-2xs cursor-pointer"
                                                        } else {
                                                            "py-1.5 rounded-lg text-[11px] font-semibold text-slate-600 hover:text-slate-900 cursor-pointer"
                                                        }
                                                        on:click=move |_| {
                                                            error_message.set(None);
                                                            email_auth_mode.set("login".to_string());
                                                            let _ = js_sys::eval("if (window.ecoTurnstile) window.ecoTurnstile.remove();");
                                                        }
                                                    >
                                                        "Ya tengo cuenta"
                                                    </button>
                                                    <button
                                                        type="button"
                                                        class=move || if is_reg_mode() {
                                                            "py-1.5 rounded-lg text-[11px] font-bold bg-white text-emerald-800 shadow-2xs cursor-pointer"
                                                        } else {
                                                            "py-1.5 rounded-lg text-[11px] font-semibold text-slate-600 hover:text-slate-900 cursor-pointer"
                                                        }
                                                        on:click=move |_| {
                                                            error_message.set(None);
                                                            email_auth_mode.set("register".to_string());
                                                            render_turnstile_if_needed();
                                                        }
                                                    >
                                                        "Crear cuenta"
                                                    </button>
                                                </div>

                                                {move || if is_reg_mode() {
                                                    view! {
                                                        <div class="space-y-2.5">
                                                            <input
                                                                type="text"
                                                                placeholder="Nombre de usuario (ej. vecino_pasco)"
                                                                class="w-full bg-white text-xs text-slate-800 placeholder-slate-400 rounded-xl px-3 py-2 border border-slate-200 focus:border-emerald-500 focus:outline-none"
                                                                prop:value=move || reg_username.get()
                                                                on:input=move |ev| reg_username.set(event_target_value(&ev))
                                                            />
                                                            <input
                                                                type="email"
                                                                placeholder="Correo electrónico"
                                                                class="w-full bg-white text-xs text-slate-800 placeholder-slate-400 rounded-xl px-3 py-2 border border-slate-200 focus:border-emerald-500 focus:outline-none"
                                                                prop:value=move || reg_email.get()
                                                                on:input=move |ev| reg_email.set(event_target_value(&ev))
                                                            />
                                                            <input
                                                                type="password"
                                                                placeholder="Contraseña (mín. 8 caracteres)"
                                                                class="w-full bg-white text-xs text-slate-800 placeholder-slate-400 rounded-xl px-3 py-2 border border-slate-200 focus:border-emerald-500 focus:outline-none"
                                                                prop:value=move || reg_password.get()
                                                                on:input=move |ev| reg_password.set(event_target_value(&ev))
                                                            />
                                                            <div id="eco-turnstile-register-container" class="flex justify-center min-h-[65px]"></div>
                                                            <button
                                                                type="button"
                                                                class="w-full py-2.5 px-4 rounded-xl font-semibold text-xs text-white bg-emerald-600 hover:bg-emerald-700 disabled:bg-slate-400 shadow-xs transition cursor-pointer"
                                                                disabled=move || is_submitting.get()
                                                                on:click=move |_| perform_citizen_register()
                                                            >
                                                                {move || if is_submitting.get() { "Creando cuenta..." } else { "Registrarme" }}
                                                            </button>
                                                        </div>
                                                    }.into_view()
                                                } else {
                                                    view! {
                                                        <div class="space-y-2.5">
                                                            <input
                                                                type="text"
                                                                placeholder="Usuario o correo electrónico"
                                                                class="w-full bg-white text-xs text-slate-800 placeholder-slate-400 rounded-xl px-3 py-2 border border-slate-200 focus:border-emerald-500 focus:outline-none"
                                                                prop:value=move || citizen_identifier.get()
                                                                on:input=move |ev| citizen_identifier.set(event_target_value(&ev))
                                                            />
                                                            <input
                                                                type="password"
                                                                placeholder="Contraseña"
                                                                class="w-full bg-white text-xs text-slate-800 placeholder-slate-400 rounded-xl px-3 py-2 border border-slate-200 focus:border-emerald-500 focus:outline-none"
                                                                prop:value=move || citizen_password.get()
                                                                on:input=move |ev| citizen_password.set(event_target_value(&ev))
                                                                on:keydown=move |ev: web_sys::KeyboardEvent| {
                                                                    if ev.key() == "Enter" {
                                                                        perform_citizen_login();
                                                                    }
                                                                }
                                                            />
                                                            <button
                                                                type="button"
                                                                class="w-full py-2.5 px-4 rounded-xl font-semibold text-xs text-white bg-slate-900 hover:bg-slate-800 disabled:bg-slate-400 shadow-xs transition cursor-pointer"
                                                                disabled=move || is_submitting.get()
                                                                on:click=move |_| perform_citizen_login()
                                                            >
                                                                {move || if is_submitting.get() { "Ingresando..." } else { "Iniciar Sesión" }}
                                                            </button>
                                                        </div>
                                                    }.into_view()
                                                }}
                                            </div>
                                        }
                                    })}
                                </div>
                            </div>

                            // ==================== PANEL 2: AUTORIDADES ====================
                            <div class=move || if !is_usuarios() { "space-y-3.5 pt-1 animate-in fade-in duration-150" } else { "hidden" }>
                                <div class="p-2.5 bg-slate-50 rounded-xl border border-slate-200/80 flex items-start space-x-2">
                                    <svg class="w-4 h-4 text-slate-500 mt-0.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                        <path d="M3 21h18"/>
                                        <path d="M5 21V7l8-4v18"/>
                                        <path d="M19 21V11l-6-4"/>
                                        <path d="M9 9h1"/>
                                        <path d="M9 13h1"/>
                                        <path d="M9 17h1"/>
                                    </svg>
                                    <p class="text-[11px] text-slate-600 font-medium leading-relaxed">
                                        "Ingreso con credenciales oficiales para fiscalizadores ambientales, Serenazgo y conductores recolectores."
                                    </p>
                                </div>

                                <div>
                                    <label class="block text-[10px] font-bold text-slate-500 uppercase tracking-wider mb-1">
                                        "Usuario Oficial"
                                    </label>
                                    <div class="relative">
                                        <input
                                            type="text"
                                            placeholder="Ej. autoridad_pasco o serenazgo_pasco"
                                            class="w-full bg-slate-50 text-xs text-slate-800 placeholder-slate-400 rounded-xl px-3 py-2.5 pl-9 border border-slate-200 focus:bg-white focus:border-slate-400 focus:outline-none focus:ring-2 focus:ring-slate-100 transition"
                                            prop:value=move || login_username.get()
                                            on:input=move |ev| login_username.set(event_target_value(&ev))
                                        />
                                        <svg class="w-4 h-4 text-slate-400 absolute left-3 top-3 pointer-events-none" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                            <path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/>
                                            <circle cx="12" cy="7" r="4"/>
                                        </svg>
                                    </div>
                                </div>

                                <div>
                                    <label class="block text-[10px] font-bold text-slate-500 uppercase tracking-wider mb-1">
                                        "Contraseña"
                                    </label>
                                    <div class="relative">
                                        <input
                                            type="password"
                                            placeholder="••••••••"
                                            class="w-full bg-slate-50 text-xs text-slate-800 placeholder-slate-400 rounded-xl px-3 py-2.5 pl-9 border border-slate-200 focus:bg-white focus:border-slate-400 focus:outline-none focus:ring-2 focus:ring-slate-100 transition"
                                            prop:value=move || login_password.get()
                                            on:input=move |ev| login_password.set(event_target_value(&ev))
                                            on:keydown=move |ev: web_sys::KeyboardEvent| {
                                                if ev.key() == "Enter" {
                                                    perform_authority_login();
                                                }
                                            }
                                        />
                                        <svg class="w-4 h-4 text-slate-400 absolute left-3 top-3 pointer-events-none" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                            <rect width="18" height="11" x="3" y="11" rx="2" ry="2"/>
                                            <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
                                        </svg>
                                    </div>
                                </div>

                                <button
                                    type="button"
                                    class="w-full mt-2 py-2.5 px-4 rounded-xl font-semibold text-xs text-white bg-slate-900 hover:bg-slate-800 disabled:bg-slate-400 shadow-xs flex items-center justify-center space-x-2 transition cursor-pointer"
                                    disabled=move || is_submitting.get()
                                    on:click=move |_| perform_authority_login()
                                >
                                    {move || if is_submitting.get() {
                                        view! {
                                            <span class="inline-block w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
                                            <span>"Iniciando sesión..."</span>
                                        }
                                    } else {
                                        view! {
                                            <svg class="w-4 h-4 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2">
                                                <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"/>
                                                <polyline points="10 17 15 12 10 7"/>
                                                <line x1="15" y1="12" x2="3" y2="12"/>
                                            </svg>
                                            <span>"Ingresar como Autoridad"</span>
                                        }
                                    }}
                                </button>
                            </div>
                        </div>

                        // Footer
                        <div class="px-6 py-3 bg-slate-50/80 border-t border-slate-100 flex items-center justify-between text-[11px] text-slate-400 shrink-0">
                            <span>"EcoAlerta · Cerro de Pasco"</span>
                            <span class="font-medium text-emerald-700 flex items-center space-x-1">
                                <svg class="w-3.5 h-3.5 text-emerald-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <rect width="18" height="11" x="3" y="11" rx="2" ry="2"/>
                                    <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
                                </svg>
                                <span>"Conexión Segura SSL"</span>
                            </span>
                        </div>
                    </div>
                </div>
            }
        })}
    }
}
