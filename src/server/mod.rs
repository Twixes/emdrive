use crate::config;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use log::*;
use std::{convert, net, str::FromStr};

async fn echo(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.uri().path(), req.method()) {
        ("/", &Method::POST) => {
            let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
            let body_string = String::from_utf8(body_bytes.into_iter().collect()).unwrap();
            Ok(Response::new(Body::from(body_string)))
        }
        ("/", _) => {
            let mut method_not_allowed = Response::default();
            *method_not_allowed.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
            Ok(method_not_allowed)
        }
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C signal handler");
    info!("💤 Shutting down gracefully");
}

/// Start server loop.
pub async fn start_server(config: config::Config) {
    let tcp_listen_address = net::SocketAddr::new(
        net::IpAddr::from_str(&config.tcp_listen_host).unwrap(),
        config.tcp_listen_port,
    );
    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, convert::Infallible>(service_fn(echo)) });

    let server = Server::bind(&tcp_listen_address)
        .serve(make_svc)
        .with_graceful_shutdown(shutdown_signal());

    info!("👂 Listening on {}...", tcp_listen_address);

    if let Err(e) = server.await {
        error!("🛑 Encountered server error: {}", e);
    };
}
