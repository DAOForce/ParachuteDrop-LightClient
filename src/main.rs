use actix_web::{middleware, web, App, HttpRequest, HttpServer};
use actix_web::web::Data;

mod routes;
mod http;

use crate::routes::health::osmosis_health;

async fn index(req: HttpRequest) -> &'static str {
    println!("REQ: {req:?}");
    "Parachute Drop!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    HttpServer::new(|| {
        let health_controller = web::scope("/health")
            .app_data(Data::new(reqwest::Client::new()))
            .service(osmosis_health);
        App::new()
            .wrap(middleware::Logger::default())
            .service(health_controller)
            .service(web::resource("/index.html").to(|| async { "Hello world!" }))
            .service(web::resource("/").to(index))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[cfg(test)]
mod tests {
    use actix_web::{body::to_bytes, dev::Service, http, test, web, App, Error};
    use crate::http::response::HealthResponse;

    use super::*;

    #[actix_web::test]
    async fn test_index() -> Result<(), Error> {
        let app = App::new().route("/", web::get().to(index));
        let app = test::init_service(app).await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = app.call(req).await?;

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = resp.into_body();
        assert_eq!(to_bytes(response_body).await?, r##"Parachute Drop!"##);

        Ok(())
    }

    #[actix_web::test]
    async fn test_osmosis_health() -> Result<(), Error> {
        let health_controller = web::scope("/health")
            .app_data(Data::new(reqwest::Client::new()))
            .service(osmosis_health);
        let app = App::new()
            .wrap(middleware::Logger::default())
            .service(health_controller);
        let app = test::init_service(app).await;

        let req = test::TestRequest::get().uri("/health/osmosis").to_request();
        let resp = app.call(req).await?;

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = resp.into_body();
        let data: HealthResponse = serde_json::from_slice(&to_bytes(response_body).await?).unwrap();
        assert_eq!(data.status, 200);
        assert_eq!(data.message, "now working");
        assert!(data.data.is_some());

        Ok(())
    }

}
