use crate::prelude::*;

use webauthn_rs_proto::{PublicKeyCredential, RequestChallengeResponse, RegisterPublicKeyCredential, CreationChallengeResponse};
use phosphor_leptos::{Icon, IconWeight, WARNING};

#[cfg(feature = "ssr")]
mod token;

#[cfg(feature = "ssr")]
mod cookie;

#[cfg(feature = "ssr")]
pub mod session;

#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    session().logout();
    Ok(())
}

#[server]
async fn is_logged_in() -> Result<bool, ServerFnError> {
    Ok(session().is_logged_in())
}

#[server]
async fn user_exists(username: String) -> Result<bool, ServerFnError> {
    Ok(Users::find()
        .filter(entity::users::Column::Username.eq(username))
        .one(&db())
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .is_some())
}

#[component]
pub fn AuthPage() -> impl IntoView {
    let is_logged_in = OnceResource::new_blocking(is_logged_in());

    view! {
        <Shell>
            <div class="flex items-center justify-center min-h-screen">
                <div class="max-w-sm w-full p-8 lg:p-16 bg-stone-100 crumpled-paper shadow-md">
                    <h1 class="text-4xl font-black text-stone-600 mb-5">"welcome!!"</h1>
                    <p class="text-stone-400 text-sm mb-7">
                        "enter your username to sign in or sign up."
                    </p>

                    <Suspense fallback=|| {}>
                        {move || Suspend::new(async move {
                            view! {
                                <Show when={move || matches!(is_logged_in.get(), Some(Ok(false)))} fallback=|| {
                                    view! {
                                        <p>
                                            "You are already logged in. To log in again, please log out first."
                                        </p>
                                        <button on:click=move |_| {
                                            spawn_local(async move {
                                                logout().await.expect("failed to log out");
                                                window().location().reload().expect("failed to reload page");
                                            });
                                        }>
                                            "sign out"
                                        </button>
                                    }
                                }>
                                    <LoginForm />
                                </Show>
                            }
                        })}
                    </Suspense>
                </div>
            </div>
        </Shell>
    }
}

#[component]
fn LoginForm() -> impl IntoView {
    let username = RwSignal::new("".to_string());
    let username_error = RwSignal::new(None::<UsernameValidationError>);

    let login: Action<_, Result<()>> = Action::new_local(|username: &String| {
        let username = username.to_owned();
        async move {
            #[cfg(feature = "hydrate")]
            match user_exists(username.clone()).await {
                Ok(true) => {
                    // Login
                    let (challenge, id) = passkey::start_login(username).await.map_err(|error| {
                        log::error!("failed to get login challenge: {error:?}");
                        anyhow::anyhow!("Failed to get login challenge")
                    })?;
                    let c_options: web_sys::CredentialRequestOptions = challenge.into();
                    let promise = window()
                        .navigator()
                        .credentials()
                        .get_with_options(&c_options)
                        .expect_throw("unable to create promise");
                    let credential = web_sys::PublicKeyCredential::from(JsFuture::from(promise).await.map_err(|error| {
                        // User probably cancelled the prompt
                        log::error!("failed to get credential: {error:?}");
                        anyhow::anyhow!("Failed to get passkey")
                    })?);
                    passkey::finish_login(id, PublicKeyCredential::from(credential)).await.map_err(|error| {
                        log::error!("failed to finish passkey login: {error:?}");
                        anyhow::anyhow!("Bad credentials")
                    })?;

                    window().location().set_href("/").expect("failed to redirect to home page");
                }
                Ok(false) => {
                    // Register
                    let ccr = passkey::start_register(username.clone()).await.map_err(|error| {
                        log::error!("failed to start passkey registration: {error:?}");
                        anyhow::anyhow!("Username already taken") // likely problem
                    })?;
                    let c_options: web_sys::CredentialCreationOptions = ccr.into();
                    let promise = window()
                        .navigator()
                        .credentials()
                        .create_with_options(&c_options)
                        .expect_throw("unable to create promise");
                    let credential = web_sys::PublicKeyCredential::from(JsFuture::from(promise).await.map_err(|error| {
                        // User probably cancelled the prompt
                        log::error!("failed to create credential: {error:?}");
                        anyhow::anyhow!("Failed to create passkey")
                    })?);
                    passkey::finish_register(username, RegisterPublicKeyCredential::from(credential)).await.map_err(|error| {
                        log::error!("failed to finish passkey registration: {error:?}");
                        anyhow::anyhow!("Failed to register passkey and/or create user")
                    })?;

                    window().location().set_href("/").expect("failed to redirect to home page");
                }
                Err(error) => {
                    log::error!("failed to check if user exists: {error:?}");
                    return Err(anyhow::anyhow!("Failed to check if user exists"));
                }
            }
            #[cfg(not(feature = "hydrate"))]
            {
                let _ = username;
            }
            Ok(())
        }
    });

    // https://web.dev/articles/passkey-form-autofill
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let controller = web_sys::AbortController::new().expect_throw("failed to create abort controller");
        let signal = controller.signal();

        spawn_local(async move {
            let cma = JsFuture::from(web_sys::PublicKeyCredential::is_conditional_mediation_available())
                .await
                .expect_throw("failed to check conditional mediation availability");
            if !cma.is_truthy() {
                log::warn!("conditional mediation is not available for passkeys");
                return;
            }

            let (challenge, id) = passkey::start_discoverable_login().await.expect_throw("failed to get login challenge");

            let c_options: web_sys::CredentialRequestOptions = challenge.into();
            c_options.set_signal(&signal);
            js_sys::Reflect::set(&c_options, &JsValue::from_str("mediation"), &JsValue::from_str("conditional")).expect_throw("failed to set mediation option");

            let promise = window()
                .navigator()
                .credentials()
                .get_with_options(&c_options)
                .expect_throw("unable to create promise");
            let Ok(credential) = JsFuture::from(promise).await else {
                if !signal.aborted() {
                    log::error!("failed to get credential, user probably cancelled the prompt");
                }
                return;
            };
            passkey::finish_discoverable_login(id, web_sys::PublicKeyCredential::from(credential).into()).await.expect_throw("failed to finish passkey login");
            window().location().set_href("/").expect("failed to redirect to home page");
        });

        struct AbortOnDrop(web_sys::AbortController);
        impl Drop for AbortOnDrop {
            fn drop(&mut self) {
                self.0.abort_with_reason(&JsValue::from_str("component unmounted"));
            }
        }
        AbortOnDrop(controller)
    });

    view! {
        <noscript>"please enable JavaScript to authenticate."</noscript>

        <div>
            <input type="text" autocomplete="username webauthn" autofocus maxlength=20 placeholder="username" bind:value=username on:change={move |_| {
                if username.get().is_empty() {
                    username_error.set(None);
                }
                username_error.set(check_username_validity(&username.get()).err());
            }} class="border border-stone-200 py-1.5 px-4 rounded-md w-full bg-white text-stone-800 placeholder-stone-400" />

            <div class="mt-1 text-xs text-red-500 min-h-5 flex items-center gap-1" aria-live="polite">
                <Show when={move || username_error.get().is_some()}>
                    <Icon icon=WARNING weight=IconWeight::Fill />
                </Show>
                {move || username_error.get().map(|e| e.to_string())}
            </div>
        </div>

        <div class="flex items-center justify-between mt-3">
            <button
                class="bg-yellow-500 hover:bg-yellow-600 text-white font-semibold py-1.5 px-4 rounded w-full flex items-center justify-center gap-2"
                on:click={move |_| { login.dispatch(username.get()); }}
                disabled={move || username.get().is_empty() || username_error.get().is_some()}
            >
                <img src="/FIDO_Passkey_mark_A_white.svg" class="w-6 h-6" />
                "continue with passkey"
            </button>
        </div>

        <div class="mt-1 text-xs text-red-500 min-h-5 flex items-center gap-1" aria-live="polite">
            <Show when={move || login.value().read().as_ref().is_some_and(|r| r.is_err())}>
                <Icon icon=WARNING weight=IconWeight::Fill />
            </Show>
            {move || login.value().read().as_ref().map(|r| r.as_ref().err().map(|e| e.to_string()))}
        </div>
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
enum UsernameValidationError {
    #[error("too short, must be at least 4 characters")]
    TooShort,
    #[error("too long, must be at most 20 characters")]
    TooLong,
    #[error("letters, numbers, underscores, and hyphens only")]
    InvalidCharacters,
    #[error("must start and end with a letter or number")]
    InvalidStartOrEnd,
    #[error("this username is not allowed")]
    Banned,
}

fn normalize_username(username: &str) -> String {
    username.trim().to_lowercase().replace('-', "_").replace('1', "l")
}

fn check_username_validity(username: &str) -> Result<(), UsernameValidationError> {
    let banlist = [
        "admin", "administrator", "root", "superuser", "test", "guest", "anon", "anonymous",
        "support", "info", "contact", "webmaster", "sysadmin", "system", "service", "starhaven", "star_haven",
    ];
    if banlist.contains(&normalize_username(username).as_str()) {
        return Err(UsernameValidationError::Banned);
    }
    if username.len() < 4 {
        return Err(UsernameValidationError::TooShort);
    }
    if username.len() > 20 {
        return Err(UsernameValidationError::TooLong);
    }
    if !username.chars().next().is_some_and(|c| c.is_ascii_alphanumeric()) ||
       !username.chars().last().is_some_and(|c| c.is_ascii_alphanumeric()) {
        return Err(UsernameValidationError::InvalidStartOrEnd);
    }
    if !username.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err(UsernameValidationError::InvalidCharacters);
    }
    Ok(())
}

pub mod passkey {
    use super::*;

    use cfg_if::cfg_if;

    cfg_if! {
        if #[cfg(feature = "ssr")] {
            use std::collections::HashMap;
            use tokio::sync::RwLock;
            use once_cell::sync::Lazy;

            use webauthn_rs::prelude::*;

            fn webauthn() -> Webauthn {
                // TODO: prod
                let rp_id = "localhost";
                let rp_origin = Url::parse("http://localhost").expect("valid rp_origin");
                WebauthnBuilder::new(rp_id, &rp_origin).expect("compatible rp_id and rp_origin")
                    .rp_name("Star Haven")
                    .allow_any_port(true)
                    .build()
                    .expect("valid configuration")
            }

            /// Map of usernames to future user uuids / passkey registration challenges. Temporary state held between start_register and finish_register calls.
            static REGISTER_CHALLENGES: Lazy<RwLock<HashMap<String, (Uuid, PasskeyRegistration)>>> = Lazy::new(|| RwLock::new(HashMap::new()));

            /// Map of random id to passkey authentication state
            static LOGIN_CHALLENGES: Lazy<RwLock<HashMap<Uuid, PasskeyAuthentication>>> = Lazy::new(|| RwLock::new(HashMap::new()));

            static DISCOVERABLE_LOGIN_CHALLENGES: Lazy<RwLock<HashMap<Uuid, DiscoverableAuthentication>>> = Lazy::new(|| RwLock::new(HashMap::new()));
        }
    }

    #[server]
    pub async fn start_register(username: String) -> Result<CreationChallengeResponse, ServerFnError> {
        let existing_credentials = match session().user().await? {
            // Registering a new passkey for an existing user
            Some(user) => {
                if user.username != username {
                    return Err(ServerFnError::new("Username does not match the logged-in user."));
                }

                Some(Passkeys::find()
                    .filter(entity::passkeys::Column::UserId.eq(user.id))
                    .all(&db())
                    .await?
                    .into_iter()
                    .map(|passkey| passkey.id.into())
                    .collect())
            }

            // Registering a new passkey for a new user
            None => {
                check_username_validity(&username)?;
                if Users::find()
                    .filter(entity::users::Column::UsernameNormalized.eq(normalize_username(&username)))
                    .one(&db())
                    .await?
                    .is_some()
                {
                    return Err(ServerFnError::new("Username already in use, please log in instead."));
                }
                None
            }
        };

        let user_id = Uuid::new_v4();
        let (ccr, skr) = webauthn().start_passkey_registration(
            user_id,
            &username,
            &username,
            existing_credentials,
        )?;
        if REGISTER_CHALLENGES.write().await.insert(username.clone(), (user_id, skr)).is_some() {
            log::warn!("overwriting existing passkey registration challenge for username: {username}");
        }
        Ok(ccr)
    }

    #[server]
    pub async fn finish_register(username: String, reg: RegisterPublicKeyCredential) -> Result<(), ServerFnError> {
        if let Some(user) = session().user().await? {
            if user.username != username {
                return Err(ServerFnError::new("Username does not match the logged-in user."));
            }
        }

        let Some((user_id, skr)) = REGISTER_CHALLENGES.write().await.remove(&username) else {
            return Err(ServerFnError::new("No registration challenge found."));
        };
        let passkey = webauthn().finish_passkey_registration(&reg, &skr)?;
        let id = passkey.cred_id().to_vec();

        // Passkeys must not be registered to this user or another user
        if Passkeys::find_by_id(id.clone())
            .one(&db())
            .await?
            .is_some()
        {
            return Err(ServerFnError::new("This passkey is already registered."));
        }
        
        db().transaction::<_, (), anyhow::Error>(|txn| {
            Box::pin(async move {
                use sea_orm::Set;

                // Use the session user or create a new user if not logged in
                let user = if let Some(user) = session().user().await? {
                    user
                } else {
                    let user = entity::users::ActiveModel {
                        id: Set(user_id),
                        username: Set(username.clone()),
                        username_normalized: Set(normalize_username(&username)),
                        ..Default::default()
                    }.insert(txn).await?;
                    session().login(&user)?;
                    user
                };

                entity::passkeys::ActiveModel {
                    id: Set(id),
                    user_id: Set(user.id),
                    data: Set(serde_json::to_value(&passkey)?),
                    ..Default::default()
                }.insert(txn).await?;

                Ok(())
            })
        }).await?;

        Ok(())
    }

    #[server]
    pub async fn start_login(username: String) -> Result<(RequestChallengeResponse, Uuid), ServerFnError> {
        let passkeys = Passkeys::find()
            .filter(entity::passkeys::Column::UserId.in_subquery(
                sea_orm::sea_query::Query::select()
                    .column(entity::users::Column::Id)
                    .from(entity::users::Entity)
                    .and_where(entity::users::Column::Username.eq(&username))
                    .to_owned()
            ))
            .all(&db())
            .await?
            .into_iter()
            .map(|passkey| serde_json::from_value::<Passkey>(passkey.data).expect("valid passkey data"))
            .collect::<Vec<Passkey>>();
    
        if passkeys.is_empty() {
            return Err(ServerFnError::new("No passkeys found for this user."));
        }

        let (challenge, auth) = webauthn().start_passkey_authentication(&passkeys)?;
        let id = Uuid::new_v4();
        LOGIN_CHALLENGES.write().await.insert(id, auth);
        Ok((challenge, id))
    }

    #[cfg(feature = "ssr")]
    async fn authenticate(authentication: AuthenticationResult) -> Result<(), ServerFnError> {
        let passkey_id = authentication.cred_id().to_vec();
        let passkey_db = Passkeys::find_by_id(passkey_id.clone())
            .one(&db())
            .await?
            .ok_or_else(|| ServerFnError::new("Passkey not found."))?;
        let mut passkey_data = serde_json::from_value::<Passkey>(passkey_db.data.clone()).expect("valid passkey data");
        let user =  Users::find_by_id(passkey_db.user_id)
            .one(&db())
            .await?
            .ok_or_else(|| ServerFnError::new("User not found."))?;

        assert!(passkey_data.update_credential(&authentication).is_some());

        let mut passkey_db: entity::passkeys::ActiveModel = passkey_db.into();
        passkey_db.data = sea_orm::Set(serde_json::to_value(&passkey_data)?);
        passkey_db.last_used_at = sea_orm::Set(Some(time::OffsetDateTime::now_utc()));
        passkey_db.update(&db()).await?;

        session().login(&user)?;
        Ok(())
    }

    #[server]
    pub async fn finish_login(id: Uuid, credential: PublicKeyCredential) -> Result<(), ServerFnError> {
        let Some(auth) = LOGIN_CHALLENGES.write().await.remove(&id) else {
            return Err(ServerFnError::new("Invalid challenge ID."));
        };

        authenticate(webauthn().finish_passkey_authentication(&credential, &auth)?).await
    }

    #[server]
    pub async fn start_discoverable_login() -> Result<(RequestChallengeResponse, Uuid), ServerFnError> {
        let (challenge, auth) = webauthn().start_discoverable_authentication()?;
        let id = Uuid::new_v4();
        DISCOVERABLE_LOGIN_CHALLENGES.write().await.insert(id, auth);
        Ok((challenge, id))
    }

    #[server]
    pub async fn finish_discoverable_login(id: Uuid, credential: PublicKeyCredential) -> Result<(), ServerFnError> {
        let Some(auth) = DISCOVERABLE_LOGIN_CHALLENGES.write().await.remove(&id) else {
            return Err(ServerFnError::new("Invalid challenge ID."));
        };

        let (user_id, _) = webauthn().identify_discoverable_authentication(&credential)?;
        let passkeys = Passkeys::find()
            .filter(entity::passkeys::Column::UserId.eq(user_id))
            .all(&db())
            .await?
            .into_iter()
            .map(|passkey| serde_json::from_value::<Passkey>(passkey.data).expect("valid passkey data").into())
            .collect::<Vec<DiscoverableKey>>();

        authenticate(webauthn().finish_discoverable_authentication(&credential, auth, &passkeys)?).await
    }
}
