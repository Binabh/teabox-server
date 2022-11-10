use hyper::{server::conn::Http, service::service_fn, Body, Method, Request, Response, StatusCode};
use std::{convert::Infallible, env};
use tokio::{fs, net::TcpListener};

#[tokio::main]
async fn main() {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:7878".to_string());
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Listening on: {}", &addr);
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            if let Err(err) = Http::new()
                .serve_connection(stream, service_fn(respond))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn respond(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            let body = fs::read_to_string("index.html").await.unwrap();
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(body))
                .unwrap())
        }
        (&Method::POST, "/") => {
            // TODO: Implement file storing and return file url in body
            Ok(Response::builder()
                .status(StatusCode::CREATED)
                .body(Body::empty())
                .unwrap())
        }
        (&Method::GET, ..) => {
            // TODO: Send file if url is present else send te response below
            let body = fs::read_to_string("404.html").await.unwrap();
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(body))
                .unwrap())
        }
        _ => {
            let body = fs::read_to_string("404.html").await.unwrap();
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(body))
                .unwrap())
        }
    }
}
