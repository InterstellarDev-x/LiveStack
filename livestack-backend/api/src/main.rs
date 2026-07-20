use poem::{EndpointExt, Route, Server, delete, get, listener::TcpListener, patch, post};
use store::Store;

use crate::{
    middleware::auth::log,
    routes::ai::chat as ai_chat,
    routes::incident::{get_user_incidents, get_website_incidents},
    routes::network_trace::network_trace,
    routes::status_page::{
        add_status_page_monitor, create_status_page, delete_status_page, get_public_status_page,
        get_status_page, get_status_pages, remove_status_page_monitor, update_status_page,
    },
    routes::user::{get_current_user, signin, signup, update_email, update_email_alerts},
    routes::website::{
        create_website, delete_website, get_website, get_website_ticks, get_website_webhook,
        get_websites_by_user, regenerate_website_webhook_secret, set_website_webhook,
        update_website,
    },
};
pub mod middleware;
pub mod routes;
pub mod types;
pub mod utils;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // fail fast so we never run with a missing signing key
    if std::env::var("JWT_SECRET").is_err() {
        return Err("JWT_SECRET must be set (e.g. in a .env file)".into());
    }

    let store_pool =
        Store::pool().map_err(|err| -> Box<dyn std::error::Error> { Box::new(err) })?;

    // fail fast: the network-trace tool can't geolocate hops without this
    let geoip_db_path = std::env::var("GEOIP_DB_PATH")
        .map_err(|_| "GEOIP_DB_PATH must be set (path to a GeoLite2-City.mmdb file)")?;
    let geoip = std::sync::Arc::new(
        maxminddb::Reader::open_readfile(&geoip_db_path)
            .map_err(|err| -> Box<dyn std::error::Error> { Box::new(err) })?,
    );

    // specify the business logic
    let app = Route::new()
        .at(
            "/website/:website_id",
            get(get_website)
                .put(update_website)
                .delete(delete_website)
                .around(log), // middleware
        )
        .at("/website", post(create_website).around(log))
        .at("/website/:website_id/ticks", get(get_website_ticks).around(log))
        .at(
            "/website/:website_id/incidents",
            get(get_website_incidents).around(log),
        )
        .at("/incidents", get(get_user_incidents).around(log))
        .at("/ai/chat", post(ai_chat).around(log))
        .at("/network-trace", post(network_trace).around(log))
        .at(
            "/website/:website_id/webhook",
            get(get_website_webhook)
                .put(set_website_webhook)
                .around(log),
        )
        .at(
            "/website/:website_id/webhook/regenerate",
            post(regenerate_website_webhook_secret).around(log),
        )
        .at("/websites", get(get_websites_by_user).around(log)) // user comes from the token
        .at("/signup", post(signup))
        .at("/signin", post(signin))
        .at("/user/email", patch(update_email).around(log))
        .at("/user/me", get(get_current_user).around(log))
        .at("/user/notifications", patch(update_email_alerts).around(log))
        .at(
            "/status-pages",
            get(get_status_pages).post(create_status_page).around(log),
        )
        .at(
            "/status-pages/:status_page_id",
            get(get_status_page)
                .put(update_status_page)
                .delete(delete_status_page)
                .around(log),
        )
        .at(
            "/status-pages/:status_page_id/monitors",
            post(add_status_page_monitor).around(log),
        )
        .at(
            "/status-pages/:status_page_id/monitors/:website_id",
            delete(remove_status_page_monitor).around(log),
        )
        // public - no auth, intentionally not wrapped in `.around(log)`
        .at("/public/status-pages/:slug", get(get_public_status_page))
        .data(store_pool)
        .data(geoip);

    Ok(Server::new(TcpListener::bind("0.0.0.0:3000"))
        .name("LiveStack Server") // give it a name to server
        .run(app) // this
        .await?)
}
