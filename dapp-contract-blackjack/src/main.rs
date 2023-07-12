use dotenv::dotenv;
use json::object;
use std::{env, error::Error};
// use tokio::sync::mpsc::{channel, Receiver, Sender};

async fn handle_inspect(
    _client: &hyper::Client<hyper::client::HttpConnector>,
    _server_addr: &str,
    req: json::JsonValue,
) -> Result<&'static str, Box<dyn std::error::Error>> {
    println!("Handling inspect");

    println!("req {:}", req);

    Ok("accept")
}

async fn handle_advance(
    _client: &hyper::Client<hyper::client::HttpConnector>,
    _server_addr: &str,
    req: json::JsonValue,
    // sender: &Sender<Item>,
) -> Result<&'static str, Box<dyn Error>> {
    println!("Handling advance");

    println!("req {:}", req);

    // let _ = sender
    //     .send(Item {
    //         request: req.dump(),
    //     })
    //     .await;

    Ok("accept")
}

async fn middleware() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting middleware sender");

    let client = hyper::Client::new();
    let server_addr = env::var("MIDDLEWARE_HTTP_SERVER_URL")?;

    let mut status = "accept";
    loop {
        println!("Sending finish");
        let response = object! {"status" => status.clone()};
        let request = hyper::Request::builder()
            .method(hyper::Method::POST)
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .uri(format!("{}/finish", &server_addr))
            .body(hyper::Body::from(response.dump()))?;
        let response = client.request(request).await?;
        println!("Received finish status {}", response.status());

        if response.status() == hyper::StatusCode::ACCEPTED {
            println!("No pending request, trying again");
        } else {
            let body = hyper::body::to_bytes(response).await?;
            let utf = std::str::from_utf8(&body)?;
            let req = json::parse(utf)?;

            let request_type = req["request_type"]
                .as_str()
                .ok_or("request_type is not a string")?;
            status = match request_type {
                "advance_state" => handle_advance(&client, &server_addr[..], req).await?,
                "inspect_state" => handle_inspect(&client, &server_addr[..], req).await?,
                &_ => {
                    eprintln!("Unknown request type");
                    "reject"
                }
            };
        }
    }
}

fn main() {
    dotenv().ok();
    println!("Starting middleware");
}
