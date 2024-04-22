use warp::Filter;
use std::error::Error;

#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String)
        .map(|name| handle_request(name).await);

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

async fn handle_request(name: String) -> Result<impl warp::Reply, warp::Rejection> {
    let greeting = format!("Hello, {}!", name);
    match get_ip().await {
        Ok(ip) => Ok(format!("{} The IP is: {}", greeting, ip)),
        Err(_) => Ok(greeting),
    }
}

async fn get_ip() -> Result<String, Error> {
    let resp = reqwest::get("http://httpbin.org/ip")
        .await?
        .text()
        .await?;
    Ok(resp)
}