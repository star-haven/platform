use axum::{
    http::HeaderValue,
    http::header::{COOKIE, SET_COOKIE},
    http::request::Parts,
};
use leptos::prelude::*;

pub fn get_cookie(key: &str) -> Option<String> {
    let request = use_context::<Parts>()?;
    let headers = &request.headers;
    let cookie = headers.get(COOKIE).and_then(|data| data.to_str().ok())?;
    get_cookie_value(cookie, key)
}

fn get_cookie_value(cookies: &str, key: &str) -> Option<String> {
    cookies.split(';').find_map(|cookie| {
        let cookie_arr = cookie.split_once('=').unwrap_or_default();
        if cookie_arr.0.trim().eq(key) && !cookie_arr.1.trim().is_empty() {
            Some(cookie_arr.1.to_string())
        } else {
            None
        }
    })
}

pub fn set_cookie(key: &str, value: &str, age: u64) {
    let Some(response) = use_context::<leptos_axum::ResponseOptions>() else {
        log::warn!("set_cookie got no ResponseOptions");
        return;
    };

    // Don't require HTTPS in debug
    #[cfg(debug_assertions)]
    let secure = "";
    #[cfg(not(debug_assertions))]
    let secure = "Secure;";

    let cookie = format!("{key}={value}; Path=/; {secure} SameSite=Lax; HttpOnly; Max-Age={age}");
    response.append_header(
        SET_COOKIE,
        HeaderValue::from_str(&cookie).expect("to create header value"),
    );
}
