extern crate futures;
extern crate hyper;

use futures::future;
use hyper::service::service_fn;
use hyper::{Body, Request, Response, Server, Uri};

use hyper::rt::{self, Future, Stream};
use hyper::Client;
use std::io::{self, Write};

fn build_target_uri(req: Request<hyper::Body>) -> Uri {
    let request_uri = req.uri();
    let mut uri = Uri::builder();
    uri.scheme("http");
    match request_uri.host() {
        Some(hostname) => {
            uri.authority(hostname);
        }
        None => {
            let host = req
                .headers()
                .get("Host")
                .expect("Sorry no host provided pal!");
            uri.authority(host.to_str().expect("header isn't a string"));
        }
    }
    uri.path_and_query(
        request_uri
            .path_and_query()
            .expect("need a pathy")
            .to_owned(),
    );
    uri.build().expect("doesn't look like a uri")
}

fn main() {
    type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;
    let addr = ([0, 0, 0, 0], 3000).into();

    fn proxy(req: Request<Body>) -> BoxFut {
        let request_uri = req.uri();
        let target_uri = build_target_uri(req);
        println!("Target URI: {}", target_uri);

        let client = Client::new();
        Box::new(
            client
                .get(target_uri)
                .map_err(|err| {
                    println!("Error: {}", err);
                    err
                })
                .and_then(|res| {
                    println!("Response: {}", res.status());
                    println!("Headers: {:#?}", res.headers());
                    future::ok(res)
                }),
        )
    };
    let server = Server::bind(&addr)
        .serve(|| service_fn(proxy))
        .map_err(|e| eprintln!("server error: {}", e));

    hyper::rt::run(server);
}
