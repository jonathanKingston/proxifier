extern crate futures;
extern crate hyper;

use futures::{future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Chunk, Request, Response, Server, Uri};

use hyper::rt::Future;
use hyper::Client;

/// Looking at a request's URI and gives us a new one
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

/// Takes a response and creates a new, modified response
fn modify_response(response: Response<Body>) -> Response<Body> {
    let (_parts, body) = response.into_parts();
    let mut dumb_thing = Vec::new();
    let mod_body = body.then(move |result| {
        match result {
            Ok(e) => {
                let bytes = e.into_bytes();
                dumb_thing.push(bytes.clone());

                //println!("{:?}", e);
                let mut me = String::from_utf8(bytes.to_vec()).unwrap();
                me = me.replace("https", "http");
                me = me.replace("Example", "Firefox!");
                Ok(Chunk::from(me))
            }
            Err(b) => Err(b),
        }
    });
    let mut builder = Response::builder();
    builder
        .body(Body::wrap_stream(mod_body))
        .expect("builder broke")
}

/// Avoids requesting to ourselves, in case someone just does a 'GET /' to the proxy as if it
/// was a server.
/// NOTE: This is a stupidity-protection, not a security mechanism.
///       Easily bypassable with 0x7f.0x1 instead of 127.0.0.1 etc.

fn is_obviously_localhost(host: &str) -> bool {
    host.contains("localhost")
        || host.starts_with("127.")
        || host.starts_with("0.")
        || host.starts_with("10.")
        || host.starts_with("172.")
        || host.starts_with("192.")
}

fn main() {
    type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;
    let addr = ([0, 0, 0, 0], 3000).into();

    fn proxy(req: Request<Body>) -> BoxFut {
        let target_uri = build_target_uri(req);
        println!("Target URI: {}", target_uri);

        // try not requesting towards outselves (but dont try too hard)
        if is_obviously_localhost(target_uri.host().unwrap()) {
            const SERVER_ERROR: &str = "<h1>500 Internal Server Error";
            let res = Response::builder()
                .status(500)
                //.header("X-Custom-Foo", "Bar")
                .body(Body::from(SERVER_ERROR))
                .unwrap();
            return Box::new(future::ok(res));
        } else {
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
                        let new_response = modify_response(res);
                        future::ok(new_response)
                    }),
            )
        }
    };
    let server = Server::bind(&addr)
        .serve(|| service_fn(proxy))
        .map_err(|e| eprintln!("server error: {}", e));
    println!("Listening on {:?}", addr);

    hyper::rt::run(server);
}
