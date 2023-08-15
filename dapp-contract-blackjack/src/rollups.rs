pub mod rollup {
    use hyper::{
        body::to_bytes, client::HttpConnector, header, Body, Client, Method, Request, Response,
        StatusCode,
    };
    use serde_json::{from_str, json, Value};
    use std::{env, error::Error, str::from_utf8, sync::Arc};
    use tokio::sync::{mpsc::Sender, Mutex};

    use crate::{
        models::game::game::Manager,
        util::json::{decode_payload, generate_message},
    };

    pub async fn rollup(
        manager: Arc<Mutex<Manager>>,
        sender: &Sender<Value>,
    ) -> Result<(), Box<dyn Error>> {
        println!("Starting loop...");

        let client = Client::new();
        let server_addr = env::var("MIDDLEWARE_HTTP_SERVER_URL")?;

        let mut status = "accept";
        loop {
            println!("Sending finish");
            let response = json!({ "status": status.clone() });
            let request = Request::builder()
                .method(Method::POST)
                .header(header::CONTENT_TYPE, "application/json")
                .uri(format!("{}/finish", &server_addr))
                .body(Body::from(response.to_string()))?;
            let response = client.request(request).await?;
            let status_response = response.status();
            println!("Receive finish status {}", &status_response);

            if status_response == StatusCode::ACCEPTED {
                println!("No pending rollup request, trying again");
            } else {
                let body = to_bytes(response).await?;
                let body = from_utf8(&body)?;
                let body = from_str::<Value>(body)?;

                let request_type = body["request_type"]
                    .as_str()
                    .ok_or("request_type is not a string")?;

                status = match request_type {
                    "advance_state" => {
                        handle_advance(&client, &server_addr[..], body, sender).await?
                    }
                    "inspect_state" => {
                        handle_inspect(manager.clone(), &client, &server_addr[..], body, sender)
                            .await?
                    }
                    &_ => {
                        eprintln!("Unknown request type");
                        "reject"
                    }
                }
            }
        }
    }

    async fn handle_inspect(
        manager: Arc<Mutex<Manager>>,
        client: &Client<HttpConnector>,
        server_addr: &str,
        body: Value,
        sender: &Sender<Value>,
    ) -> Result<&'static str, Box<dyn Error>> {
        println!("Handling inspect");

        println!("body {:}", &body);

        // sender.send(body).await?;
        // handle_request_action(&body, manager, false).await?;

        let payload = get_payload_from_root(&body).ok_or("Invalid payload")?;
        let action = get_from_payload_action(&payload);
        match action.as_deref() {
            Some("show_games") => {
                let manager = manager.lock().await;
                let games = manager.show_games_id_available();

                let response = json!({
                    "games": games,
                });
                let json_as_hex = hex::encode(response.to_string());
                let report = json!({ "payload": format!("0x{}", json_as_hex) });
                println!("Report: {:}", report);
                let _ = send_report(report.clone()).await;
            }
            _ => Err("Invalid inspect")?,
        };

        Ok("accept")
    }

    async fn handle_advance(
        client: &Client<HttpConnector>,
        server_addr: &str,
        body: Value,
        sender: &Sender<Value>,
    ) -> Result<&'static str, Box<dyn Error>> {
        println!("Handling advance");

        println!("body {:}", &body);

        sender.send(body).await?;

        Ok("accept")
    }

    pub(crate) async fn send_report(
        report: Value,
    ) -> Result<&'static str, Box<dyn std::error::Error>> {
        let server_addr = std::env::var("ROLLUP_HTTP_SERVER_URL")?;
        let client = hyper::Client::new();
        let req = hyper::Request::builder()
            .method(hyper::Method::POST)
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .uri(format!("{}/report", server_addr))
            .body(hyper::Body::from(report.to_string()))?;

        let _ = client.request(req).await?;
        Ok("accept")
    }

    fn get_payload_from_root(root: &Value) -> Option<Value> {
        let root = root.as_object()?;
        let root = root.get("data")?.as_object()?;
        let payload = root.get("payload")?.as_str()?;
        let payload = decode_payload(payload)?;
        Some(payload)
    }

    fn get_from_payload_action(payload: &Value) -> Option<String> {
        let input = payload.get("input")?.as_object()?;
        let action = input.get("action")?.as_str()?;
        Some(action.to_owned())
    }
}
