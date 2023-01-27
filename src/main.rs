use base64ct::{Base64Url, Encoding};
use hyper::{
    header::CONTENT_TYPE, server::conn::Http, service::service_fn, Body, Method, Request, Response,
    StatusCode,
};
use multer::Multipart;
use sha2::{Digest, Sha256};
use std::{convert::Infallible, env, path::PathBuf};
use tokio::{fs, io::AsyncWriteExt, net::TcpListener};
// TODO: move these as environment variables
const UPLOAD_FOLDER: &str = "uploads";
const TEMPLATE_FOLDER: &str = "templates";
const LOCAL_ADDRESS: &str = "127.0.0.1:7878";
const WEB_ADDRESS: &str = "http://127.0.0.1:7878/";
#[tokio::main]
async fn main() {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| LOCAL_ADDRESS.to_string());
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
            let file_path = PathBuf::from(TEMPLATE_FOLDER).join("index.html");
            let body = fs::read_to_string(file_path).await.unwrap();
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(body))
                .unwrap())
        }
        (&Method::POST, "/") => {
            // Extract the `multipart/form-data` boundary from the headers.
            let boundary = req
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|ct| ct.to_str().ok())
                .and_then(|ct| multer::parse_boundary(ct).ok());

            // Send `BAD_REQUEST` status if the content-type is not multipart/form-data.
            if boundary.is_none() {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from("BAD REQUEST"))
                    .unwrap());
            }

            // Process the multipart
            let res = process_multipart(req.into_body(), boundary.unwrap()).await;
            match res {
                Ok(url) => Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(url))
                    .unwrap()),
                Err(err) => Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(format!("INTERNAL SERVER ERROR: {}", err)))
                    .unwrap()),
            }
        }
        (&Method::GET, ..) => {
            let file_path = PathBuf::from(UPLOAD_FOLDER).join(&req.uri().path()[1..]);
            if let Ok(contents) = tokio::fs::read(file_path).await {
                let body = contents.into();
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(body)
                    .unwrap());
            }
            let file_path = PathBuf::from(TEMPLATE_FOLDER).join("404.html");

            let body = fs::read_to_string(file_path).await.unwrap();

            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(body))
                .unwrap())
        }
        _ => {
            let file_path = PathBuf::from(TEMPLATE_FOLDER).join("404.html");
            let body = fs::read_to_string(file_path).await.unwrap();
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(body))
                .unwrap())
        }
    }
}

async fn process_multipart(body: Body, boundary: String) -> multer::Result<String> {
    let mut multipart = Multipart::new(body, boundary);
    let mut file_url = String::from(WEB_ADDRESS);
    while let Some(mut field) = multipart.next_field().await? {
        let mut chunk_list = vec![];
        print!("{:?}",field);
        while let Some(field_chunk) = field.chunk().await? {
            chunk_list.push(field_chunk);
        }
        let data = chunk_list.concat();
        let hash = Sha256::digest(&data);
        const BUF_SIZE: usize = 256;
        let mut enc_buf = [0u8; BUF_SIZE];
        let encoded = Base64Url::encode(&hash, &mut enc_buf).unwrap();
        let filename = &encoded[..6];
        let file_path = PathBuf::from(UPLOAD_FOLDER).join(&filename);
        let mut file = fs::File::create(file_path).await.unwrap();
        file.write_all(&data).await.unwrap();
        file.flush().await.unwrap();
        file_url.push_str(&filename);
        file_url.push_str("\n");
    }

    Ok(file_url)
}
