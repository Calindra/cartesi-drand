pub mod common {
    use httptest::{
        all_of,
        matchers::{contains, key, request, url_decoded, Matcher},
        responders::{json_encoded, Responder},
        Expectation, ServerPool,
    };
    use hyper::{body, Request};
    use serde_json::Value;
    use std::{env::set_var, sync::Once};

    use crate::util::json::generate_message;

    static BIND_SERVER: Once = Once::new();
    static SERVER_POOL: ServerPool = ServerPool::new(2);

    pub async fn setup_hit_random() -> impl Drop {
        let message = generate_message(Value::from("blackjack"));

        add_expectation(
            all_of![
                request::method_path(hyper::Method::POST.as_str(), "/random"),
                request::query(url_decoded(contains(key("timestamp"))))
            ],
            json_encoded(message),
        )
        .await
    }

    pub async fn add_expectation(
        matcher: impl Matcher<Request<body::Bytes>> + 'static,
        responder: impl Responder + 'static,
    ) -> impl Drop {
        let server = SERVER_POOL.get_server();

        server.expect(
            Expectation::matching(matcher)
                .times(1..)
                .respond_with(responder),
        );

        let url = server.url_str("");

        println!("Server listening on {}", &url);

        BIND_SERVER.call_once(|| {
            set_var("MIDDLEWARE_HTTP_SERVER_URL", &url);
        });

        assert!(std::env::var("MIDDLEWARE_HTTP_SERVER_URL").is_ok());

        server
    }
}
