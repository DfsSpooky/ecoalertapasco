use web_sys::window;
use leptos::{SignalGet, SignalSet};
use crate::models::AuthUser;

const KEY_TOKEN: &str = "eco_auth_token";
const KEY_USERNAME: &str = "eco_auth_username";
const KEY_IS_STAFF: &str = "eco_auth_is_staff";
const KEY_EMAIL: &str = "eco_auth_email";
const KEY_PHOTO_URL: &str = "eco_auth_photo_url";
const KEY_FULL_NAME: &str = "eco_auth_full_name";
const KEY_LAST_REPORT_TS: &str = "eco_last_report_ts";
const KEY_REMEMBERED_GOOGLE_USER: &str = "eco_remembered_google_user";

pub fn save_remembered_google_user(user: &AuthUser) {
    if let Some(win) = window() {
        if let Ok(Some(storage)) = win.local_storage() {
            if let Ok(serialized) = serde_json::to_string(user) {
                let _ = storage.set_item(KEY_REMEMBERED_GOOGLE_USER, &serialized);
            }
        }
    }
}

pub fn load_remembered_google_user() -> Option<AuthUser> {
    let win = window()?;
    let storage = win.local_storage().ok()??;
    if let Ok(Some(data_str)) = storage.get_item(KEY_REMEMBERED_GOOGLE_USER) {
        if let Ok(u) = serde_json::from_str::<AuthUser>(&data_str) {
            return Some(u);
        }
    }
    // Fallback: Check if individual user keys were stored from previous sessions
    let username = storage.get_item(KEY_USERNAME).ok()??;
    let email = storage.get_item(KEY_EMAIL).ok()?.filter(|e| !e.is_empty());
    let photo_url = storage.get_item(KEY_PHOTO_URL).ok()?.filter(|p| !p.is_empty());
    let full_name = storage.get_item(KEY_FULL_NAME).ok()?.filter(|f| !f.is_empty());
    let token = storage.get_item(KEY_TOKEN).ok()?.unwrap_or_default();

    if email.is_some() || photo_url.is_some() || full_name.is_some() {
        Some(AuthUser {
            token,
            username,
            email,
            is_staff: false,
            photo_url,
            full_name,
        })
    } else {
        None
    }
}

pub fn clear_remembered_google_user() {
    if let Some(win) = window() {
        if let Ok(Some(storage)) = win.local_storage() {
            let _ = storage.remove_item(KEY_REMEMBERED_GOOGLE_USER);
        }
    }
}

pub fn save_auth_session(user: &AuthUser) {
    if let Some(win) = window() {
        if let Ok(Some(storage)) = win.local_storage() {
            let _ = storage.set_item(KEY_TOKEN, &user.token);
            let _ = storage.set_item(KEY_USERNAME, &user.username);
            let _ = storage.set_item(KEY_IS_STAFF, if user.is_staff { "true" } else { "false" });
            if let Some(ref email) = user.email {
                let _ = storage.set_item(KEY_EMAIL, email);
            }
            if let Some(ref photo) = user.photo_url {
                let _ = storage.set_item(KEY_PHOTO_URL, photo);
            }
            if let Some(ref full_name) = user.full_name {
                let _ = storage.set_item(KEY_FULL_NAME, full_name);
            }
        }
    }
    // Remember Google / citizen user details for fast one-click re-login
    if user.email.is_some() || user.photo_url.is_some() || !user.is_staff {
        save_remembered_google_user(user);
    }
}

pub fn load_auth_session() -> Option<AuthUser> {
    let win = window()?;
    let storage = win.local_storage().ok()??;
    let token = storage.get_item(KEY_TOKEN).ok()??;
    if token.trim().is_empty() {
        return None;
    }
    let username = storage
        .get_item(KEY_USERNAME)
        .ok()?
        .unwrap_or_else(|| "Ciudadano".to_string());
    let is_staff = storage
        .get_item(KEY_IS_STAFF)
        .ok()?
        .map(|v| v == "true")
        .unwrap_or(false);
    let email = storage.get_item(KEY_EMAIL).ok()?.filter(|e| !e.is_empty());
    let photo_url = storage.get_item(KEY_PHOTO_URL).ok()?.filter(|p| !p.is_empty());
    let full_name = storage.get_item(KEY_FULL_NAME).ok()?.filter(|f| !f.is_empty());

    Some(AuthUser {
        token,
        username,
        email,
        is_staff,
        photo_url,
        full_name,
    })
}

pub fn clear_auth_session() {
    if let Some(win) = window() {
        if let Ok(Some(storage)) = win.local_storage() {
            let _ = storage.remove_item(KEY_TOKEN);
            let _ = storage.remove_item(KEY_USERNAME);
            let _ = storage.remove_item(KEY_IS_STAFF);
            let _ = storage.remove_item(KEY_EMAIL);
            let _ = storage.remove_item(KEY_PHOTO_URL);
            let _ = storage.remove_item(KEY_FULL_NAME);
        }
    }
}

pub fn logout_and_clear_session(current_user: leptos::RwSignal<Option<AuthUser>>) {
    if let Some(user) = current_user.get() {
        let tok = user.token.clone();
        leptos::spawn_local(async move {
            let _ = gloo_net::http::Request::post(&crate::api::logout_url())
                .header("Authorization", &format!("Token {}", tok))
                .send()
                .await;
        });
    }
    clear_auth_session();
    current_user.set(None);
}

pub fn get_last_report_time() -> Option<f64> {
    let win = window()?;
    let storage = win.local_storage().ok()??;
    let val_str = storage.get_item(KEY_LAST_REPORT_TS).ok()??;
    val_str.parse::<f64>().ok()
}

pub fn set_last_report_time(ts_ms: f64) {
    if let Some(win) = window() {
        if let Ok(Some(storage)) = win.local_storage() {
            let _ = storage.set_item(KEY_LAST_REPORT_TS, &ts_ms.to_string());
        }
    }
}
