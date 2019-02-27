extern crate futures;
extern crate hyper;

use futures::{future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Request, Response, Server, Uri, Chunk};

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

/// Rewrites all case-sensitive occurences of the string
/// 'https' to 'http'.
/*
fn https_to_http(body: Body) -> Body {
    // TODO: actually modify it :)
    body.then(|result| {
        match result {
            Ok(e) => Ok(e),
            Err(b) => Err(b),
        }
    }).into()
}*/
/*
/// Takes a response and creates a new, modified response
fn modify_response(response: Response<Body>) -> Response<Body> {
    //let (parts, body) = response.into_parts();

    //let new_body = https_to_http(body);

    response.then(|result| {
        match result {
            Ok(e) => Ok(e),
            Err(b) => Err(b),
        }
    }).collect2()

    //Response::from_parts(parts, new_body)
}*/

fn main() {
    type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;
    let addr = ([0, 0, 0, 0], 3000).into();

    fn proxy(req: Request<Body>) -> BoxFut {
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
                    let (parts, body) = res.into_parts();
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
                            },
                            Err(b) => Err(b),
                        }
                    });
                    let mut builder = Response::builder();
                    let res = builder.body(Body::wrap_stream(mod_body)).expect("builder broke");
                    future::ok(res)
                }),
        )
    };
    let server = Server::bind(&addr)
        .serve(|| service_fn(proxy))
        .map_err(|e| eprintln!("server error: {}", e));

    hyper::rt::run(server);
}
