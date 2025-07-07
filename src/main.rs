#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use sea_orm::Database;
    use star_haven_platform::app::*;
    use migration::{Migrator, MigratorTrait};

    femme::start();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL to be set");
    let db = Database::connect(db_url).await.expect("to be able to connect to database");
    Migrator::up(&db, None).await.expect("to migrate the database");
    log::info!("database ok");

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            move || {
                provide_context(db.clone());
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            }
        )
        .nest_service("/assets", tower_http::services::ServeDir::new(star_haven_platform::static_assets_dir()))
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log::info!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
