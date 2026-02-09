// test/api/helpers

// dependencies
use rama::Layer;
use rama::Service;
use rama::http::layer::trace::TraceLayer;
use rama::http::service::web::Router;
use rama::http::{Body, Request, Response};
use rusty_checkers_server_lib::config::get_configuration;
use rusty_checkers_server_lib::startup::Application;
use rusty_checkers_server_lib::state::AppState;
use rusty_checkers_server_lib::telemetry::{get_subscriber, init_subscriber, make_request_span};
use std::sync::LazyLock;

static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub router: Router<AppState>,
}

pub async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);

    let _configuration = get_configuration().expect("Failed to read configuration.");

    let state = AppState::new();
    let router = Application::build_app_router(state);

    TestApp { router }
}

impl TestApp {
    pub async fn serve(self, request: Request) -> Response {
        let service = TraceLayer::new_for_http()
            .make_span_with(make_request_span)
            .into_layer(self.router);

        let response = service
            .serve(request)
            .await
            .expect("Failed to start service.");

        response.map(Body::new)
    }

    pub fn build_request(&self, uri: &str, body: Option<&str>) -> Request {
        let request_body = match body {
            Some(b) => Body::from(b.to_owned()),
            None => Body::empty(),
        };

        Request::builder()
            .uri(uri)
            .body(request_body)
            .expect("Failed to build request.")
    }
}
