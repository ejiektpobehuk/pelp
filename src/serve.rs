use core::panic;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::process;
use std::thread;

use axum::{http, Router};
use notify::{
    event::{AccessKind, AccessMode, EventKind, RemoveKind},
    Event, Watcher,
};
use rand::RngExt;
use tower::layer::util::Stack;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_livereload::LiveReloadLayer;

// TODO: proxy serve for embedding sites that don't allow embedding

async fn internal_serve(source_path: PathBuf, output_path: PathBuf, addr: Option<SocketAddr>) {
    let livereload = LiveReloadLayer::new();
    let reloader = livereload.reloader();
    let mut source_dir = source_path.clone();
    source_dir.pop();
    let app = Router::new()
        .nest_service("/", ServeDir::new(&source_dir))
        .layer(livereload)
        .layer(no_cache_layer());

    let mut watcher = notify::recommended_watcher(move |res| {
        match res {
            Ok(event) => {
                match event {
                    Event {
                        kind: EventKind::Access(AccessKind::Close(AccessMode::Write)),
                        paths,
                        ..
                    } => {
                        if paths.contains(&output_path) {
                            reloader.reload();
                        };
                    }
                    Event {
                        kind: EventKind::Remove(RemoveKind::File),
                        paths,
                        ..
                    } => {
                        // Vim saves file in a special way and it breaks stuff
                        //reloader.reload()
                    }
                    Event { kind, paths, .. } => {
                        //println!("Unwanted chage: {:?} {:?}", kind, paths)
                    }
                }
            }
            Err(e) => panic!("Unable to watch for file changes: {}", e),
        }
    })
    .unwrap();
    watcher
        .watch(&source_dir, notify::RecursiveMode::NonRecursive)
        .unwrap();

    // TODO: open a browser
    // TODO: open presentation at root
    tracing_subscriber::fmt::init();
    let (listener, addr) = match addr {
        Some(addr) => (tokio::net::TcpListener::bind(addr).await.unwrap(), addr),
        None => {
            let addr: std::net::SocketAddr =
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
            match tokio::net::TcpListener::bind(addr).await {
                Ok(listener) => (listener, addr),
                Err(e) => {
                    eprintln!("Unable to start listening at http://{}/.\n\tError: {}\n\tTrying another port.", addr, e);
                    let random_port = rand::rng().random_range(1025..=65535);
                    let addr: std::net::SocketAddr =
                        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), random_port);
                    match tokio::net::TcpListener::bind(addr).await {
                        Ok(listener) => (listener, addr),
                        Err(e) => {
                            eprintln!("Unable to start listening at a random port {}.", e);
                            process::exit(13)
                        }
                    }
                }
            }
        }
    };

    println!("listening on: http://{}/", addr);
    axum::serve(listener, app).await.unwrap();
}

pub fn serve(source_path: PathBuf, output_path: PathBuf) {
    let addr = None;
    thread::spawn(move || {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(internal_serve(source_path, output_path, addr));
    });
}

type Srhl = SetResponseHeaderLayer<http::HeaderValue>;

fn no_cache_layer() -> Stack<Srhl, Stack<Srhl, Srhl>> {
    Stack::new(
        SetResponseHeaderLayer::overriding(
            http::header::CACHE_CONTROL,
            http::HeaderValue::from_static("no-cache, no-store, must-revalidate"),
        ),
        Stack::new(
            SetResponseHeaderLayer::overriding(
                http::header::PRAGMA,
                http::HeaderValue::from_static("no-cache"),
            ),
            SetResponseHeaderLayer::overriding(
                http::header::EXPIRES,
                http::HeaderValue::from_static("0"),
            ),
        ),
    )
}
