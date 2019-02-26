extern crate futures;
extern crate hyper;

use futures::future;
use hyper::service::service_fn;
use hyper::{Body, Request, Response, Server, Uri};

use hyper::rt::{self, Future, Stream};
use hyper::Client;
use std::io::{self, Write};

static TEXT: &str = "Hello, World!";

fn main() {
    type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;
    let addr = ([0, 0, 0, 0], 3000).into();

    fn proxy(req: Request<Body>) -> BoxFut {
        println!("a {:?}", req.uri().host());
        let mut uri = Uri::builder();
        uri.scheme("https");
        match req.uri().host() {
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
        uri.path_and_query(req.uri().path_and_query().expect("need a pathy").to_owned());
        let request_uri = uri.build().expect("doesn't look like a uri");
        println!("b {:?}", request_uri.to_string());
        //rt::run(rt::lazy(|| {
        let client = Client::new();

        Box::new(client.get(request_uri).and_then(|res| {
            println!("Response: {}", res.status());
            println!("Headers: {:#?}", res.headers());
            future::ok(res)
        }))
        /*
        .map(|res| {
            println!("Response: {}", res.status());
        })
        .map_err(|err| {
            println!("Error: {}", err);
        })*/
        //}));
        //Box::new(future::ok(Response::new(Body::from(request_uri.to_string()))))
    };
    let server = Server::bind(&addr)
        .serve(|| service_fn(proxy))
        .map_err(|e| eprintln!("server error: {}", e));

    hyper::rt::run(server);
}
