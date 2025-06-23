use leptos::prelude::*;
use uuid::Uuid;

#[cfg(feature = "ssr")]
mod token;

#[cfg(feature = "ssr")]
mod cookie;

#[cfg(feature = "ssr")]
mod session;

#[server]
pub async fn login() -> Result<(), ServerFnError> {
    match session::session().login() {
        Ok(_) => leptos_axum::redirect("/"),
        Err(error) => {
            log::error!("error during login: {error}");
            let response = expect_context::<leptos_axum::ResponseOptions>();
            response.set_status(http::StatusCode::UNAUTHORIZED);
        }
    }
    Ok(())
}

#[component]
pub fn LoginForm() -> impl IntoView {
    let login = ServerAction::<Login>::new();

    view! {
        <Await future=get_session_uuid() let:uuid>
            <ActionForm action=login>
                <input type="submit" value="Login" />
            </ActionForm>
            <p>You are logged in as: {format!("{:?}", *uuid)}</p>
        </Await>
    }
}

#[server]
async fn get_session_uuid() -> Result<Option<Uuid>, ServerFnError> {
    Ok(session::session().uuid())
}
