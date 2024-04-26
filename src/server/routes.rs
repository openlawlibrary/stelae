//! A central place to register App routes.
use actix_web::web;

use super::{api::versions::versions, app::GlobalState};

/// Central place to register _api/ routes.
pub fn register_api<T: GlobalState + Clone + 'static>(cfg: &mut web::ServiceConfig, state: &T) {
    cfg.service(
        web::scope("/_api").service(
            web::scope("/versions")
                .service(
                    web::resource(
                        "/_publication/{publication}/_compare/{date}/{compare_date}/{path:.*}",
                    )
                    .to(versions),
                )
                .service(web::resource("/_publication/{publication}/_date/{date}").to(versions))
                .service(web::resource("/_publication/{publication}").to(versions))
                .service(web::resource("/_publication/{publication}/{path:.*}").to(versions))
                .service(web::resource("/_compare/{date}/{compare_date}/{path:.*}").to(versions))
                .service(web::resource("/_date/{date}/{path:.*}").to(versions))
                .service(web::resource("/{path:.*}").to(versions)),
        ),
    )
    .app_data(web::Data::new(state.clone()));
}
