#[cfg(test)]
pub mod common {
    use httptest::{
        all_of,
        matchers::{contains, key, request, url_decoded, Matcher},
        responders::{json_encoded, status_code, Responder},
        Expectation, ServerPool,
    };
    use hyper::{body, Request};
    use serde_json::Value;
    use std::{
        env::{set_var, var},
        sync::Once,
    };

    use crate::util::json::generate_message;

    static BIND_SERVER: Once = Once::new();
    static SERVER_POOL: ServerPool = ServerPool::new(1);

    pub async fn setup_change_key() -> impl Drop {
        add_expectation(
            all_of![request::method_path(
                hyper::Method::PUT.as_str(),
                "/update_drand_config"
            ),],
            status_code(204),
        )
        .await
    }

    pub async fn setup_dont_change_key() -> impl Drop {
        let server = SERVER_POOL.get_server();

        server.expect(
            Expectation::matching(request::method_path(
                hyper::Method::PUT.as_str(),
                "/update_drand_config",
            ))
            .times(0)
            .respond_with(status_code(500)),
        );

        let url = server.url_str("");

        BIND_SERVER.call_once(|| {
            set_var("MIDDLEWARE_HTTP_SERVER_URL", &url);
            set_var("ROLLUP_HTTP_SERVER_URL", &url);
        });

        assert!(var("MIDDLEWARE_HTTP_SERVER_URL").is_ok());
        assert!(var("ROLLUP_HTTP_SERVER_URL").is_ok());

        println!("Server listening on {}", &url);

        server
    }

    pub async fn setup_hit_random() -> impl Drop {
        let message = generate_message(Value::from("blackjack"));

        add_expectation(
            all_of![
                request::method_path(hyper::Method::GET.as_str(), "/random"),
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

        BIND_SERVER.call_once(|| {
            set_var("MIDDLEWARE_HTTP_SERVER_URL", &url);
            set_var("ROLLUP_HTTP_SERVER_URL", &url);
        });

        assert!(var("MIDDLEWARE_HTTP_SERVER_URL").is_ok());
        assert!(var("ROLLUP_HTTP_SERVER_URL").is_ok());

        println!("Server listening on {}", &url);

        server
    }
}
