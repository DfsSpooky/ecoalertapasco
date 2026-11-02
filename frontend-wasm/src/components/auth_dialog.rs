use leptos::*;
use crate::api::{login_url, google_auth_url, google_auth_config_url};
use crate::models::{AuthUser, LoginPayload, GoogleAuthPayload, GoogleAuthConfig};
use crate::storage::save_auth_session;

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

    let is_submitting = create_rw_signal(false);
    let error_message = create_rw_signal(None::<String>);

    // Google Auth State
    let google_config = create_rw_signal(None::<GoogleAuthConfig>);

    // Fetch Google configuration when dialog is opened
    create_effect(move |_| {
        if show_dialog.get() {
            error_message.set(None);
            spawn_local(async move {
                let req = gloo_net::http::Request::get(&google_auth_config_url()).send().await;
                if let Ok(resp) = req {
                    if resp.ok() {
                        if let Ok(cfg) = resp.json::<GoogleAuthConfig>().await {
                            let cid = cfg.client_id.clone();
                            google_config.set(Some(cfg));
                            if !cid.trim().is_empty() {
                                let _ = js_sys::eval(&format!("window.ecoGoogleAuth.init('{}')", cid));
                                let _ = js_sys::eval("setTimeout(function() { window.ecoGoogleAuth.renderButton('google-gis-btn-container'); }, 150)");
                            }
                        }
                    }
                }
            });
        }
    });

    let close_dialog = move |_| {
        error_message.set(None);
        show_dialog.set(false);
    };

    let switch_tab = move |tab: &'static str| {
        error_message.set(None);
        active_tab.set(tab.to_string());
        if tab == "usuarios" {
            let _ = js_sys::eval("setTimeout(function() { if (window.ecoGoogleAuth) window.ecoGoogleAuth.renderButton('google-gis-btn-container'); }, 60)");
        }
    };

    // Handle Google Login & Registration Execution
    let perform_google_login = {
        let current_user = current_user.clone();
        let show_dialog = show_dialog.clone();
        let is_submitting = is_submitting.clone();
        let error_message = error_message.clone();

        move |credential: String| {
            is_submitting.set(true);
            error_message.set(None);

            let current_user = current_user.clone();
            let show_dialog = show_dialog.clone();
            let is_submitting = is_submitting.clone();
            let error_message = error_message.clone();

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
                                    current_user.set(Some(user));
                                    show_dialog.set(false);
                                } else {
                                    error_message.set(Some("Respuesta inesperada al autenticar con Google.".to_string()));
                                }
                            } else {
                                if let Ok(err_val) = resp.json::<serde_json::Value>().await {
                                    if let Some(err_str) = err_val.get("error").and_then(|v| v.as_str()) {
                                        error_message.set(Some(err_str.to_string()));
                                    } else {
                                        error_message.set(Some("No se pudo completar el acceso con Google.".to_string()));
                                    }
                                } else {
                                    error_message.set(Some("Token de Google rechazado por el servidor.".to_string()));
                                }
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

    // Handle Authority Login
    let perform_login = move || {
        let u = login_username.get();
        let p = login_password.get();

        if u.trim().is_empty() || p.trim().is_empty() {
            error_message.set(Some("Por favor completa tu usuario y contraseña de autoridad.".to_string()));
            return;
        }

        is_submitting.set(true);
        error_message.set(None);

        spawn_local(async move {
            let payload = LoginPayload {
                username: &u,
                password: &p,
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
                                login_username.set("".to_string());
                                login_password.set("".to_string());
                                show_dialog.set(false);
                            } else {
                                error_message.set(Some("Respuesta inesperada del servidor.".to_string()));
                            }
                        } else {
                            let status = resp.status();
                            if status == 429 {
                                error_message.set(Some("Demasiados intentos de acceso fallidos. Por seguridad, espera un momento antes de reintentar.".to_string()));
                            } else if let Ok(err_val) = resp.json::<serde_json::Value>().await {
                                if let Some(err_str) = err_val.get("error").and_then(|v| v.as_str()) {
                                    error_message.set(Some(err_str.to_string()));
                                } else if let Some(detail) = err_val.get("detail").and_then(|v| v.as_str()) {
                                    error_message.set(Some(detail.to_string()));
                                } else {
                                    error_message.set(Some(format!("Error de autenticación ({})", status)));
                                }
                            } else {
                                error_message.set(Some("Credenciales oficiales inválidas. Verifica usuario y contraseña.".to_string()));
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

    view! {
        {move || show_dialog.get().then(|| {
            let is_usuarios = move || active_tab.get() == "usuarios";
            let google_login_fn = perform_google_login.clone();

            view! {
                <div class="fixed inset-0 bg-slate-900/60 backdrop-blur-sm flex items-end sm:items-center justify-center p-0 sm:p-4 z-50 animate-in fade-in duration-150">
                    <div class="bg-white w-full max-w-md rounded-t-3xl sm:rounded-2xl border border-slate-200/90 shadow-[0_24px_64px_rgba(0,0,0,0.22)] overflow-hidden max-h-[92vh] flex flex-col">
                        // Hidden button for GIS dispatch callback
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
                                    <p class="text-[11px] text-slate-500 font-medium">"Vigilancia Ambiental · Cerro de Pasco"</p>
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
                                    <span>"Usuarios"</span>
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

                            // ==================== PANEL 1: USUARIOS (EXCLUSIVO GOOGLE) ====================
                            <div class=move || if is_usuarios() { "space-y-4 pt-1 animate-in fade-in duration-150" } else { "hidden" }>
                                <div class="p-4 rounded-2xl bg-gradient-to-br from-emerald-50/80 to-teal-50/40 border border-emerald-200/80 space-y-3 shadow-2xs">
                                    <div class="flex items-start space-x-2.5">
                                        <div class="w-7 h-7 rounded-lg bg-emerald-600/10 text-emerald-700 flex items-center justify-center shrink-0 mt-0.5">
                                            <svg class="w-4 h-4 text-emerald-700" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
                                            </svg>
                                        </div>
                                        <div>
                                            <h3 class="text-xs font-bold text-emerald-950">"Acceso Ciudadano"</h3>
                                            <p class="text-[11px] text-emerald-800 leading-relaxed mt-0.5">
                                                "Inicia sesión o regístrate en un clic con tu cuenta de Google para reportar y seguir alertas ambientales."
                                            </p>
                                        </div>
                                    </div>

                                    // Official GIS native button container (rendered if Client ID is configured)
                                    <div id="google-gis-btn-container" class="w-full flex justify-center empty:hidden min-h-[42px]"></div>

                                    // Fallback button shown only if Google GIS is loading or needs prompt
                                    <button
                                        type="button"
                                        class=move || {
                                            let has_cid = google_config
                                                .get()
                                                .map(|c| !c.client_id.trim().is_empty())
                                                .unwrap_or(false);
                                            if has_cid {
                                                "hidden"
                                            } else {
                                                "w-full py-2.5 px-4 rounded-xl font-medium text-xs text-slate-700 bg-white hover:bg-slate-50 border border-slate-300 hover:border-slate-400 shadow-xs flex items-center justify-center space-x-3 transition cursor-pointer active:scale-[0.99] disabled:opacity-50"
                                            }
                                        }
                                        disabled=move || is_submitting.get()
                                        on:click=move |_| {
                                            let has_cid = google_config
                                                .get()
                                                .map(|c| !c.client_id.trim().is_empty())
                                                .unwrap_or(false);

                                            if has_cid {
                                                let _ = js_sys::eval("window.ecoGoogleAuth.prompt()");
                                            } else {
                                                error_message.set(Some(
                                                    "El servicio de Google se está conectando. Por favor espera un momento o revisa tu conexión.".to_string()
                                                ));
                                            }
                                        }
                                    >
                                        <svg class="w-4 h-4 shrink-0" viewBox="0 0 24 24">
                                            <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
                                            <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                                            <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-2.09-.35-2.09s.13-1.43.35-2.09V7.06H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.94l2.85-2.22.81-.63z"/>
                                            <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.06l3.66 2.84c.87-2.6 3.3-4.52 6.16-4.52z"/>
                                        </svg>
                                        <span class="font-semibold text-slate-800">
                                            "Continuar con Google"
                                        </span>
                                    </button>
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
                                        "Ingreso con credenciales oficiales asignadas a fiscalizadores y personal de la Municipalidad de Pasco."
                                    </p>
                                </div>

                                <div>
                                    <label class="block text-[10px] font-bold text-slate-500 uppercase tracking-wider mb-1">
                                        "Usuario Oficial"
                                    </label>
                                    <div class="relative">
                                        <input
                                            type="text"
                                            placeholder="Ej. autoridad_pasco"
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
                                                    perform_login();
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
                                    on:click=move |_| perform_login()
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
