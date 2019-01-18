#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;
use clap;
use clap::Arg;
use futures::future::{done, ok, Future};
use futures::stream::Stream;
use http::StatusCode;
use hyper::service::service_fn;
use hyper::Client;
use hyper::{Body, Request, Response, Server};
use log::{debug, error, info, warn};
use std::env;
mod exporter_error;
use crate::exporter_error::ExporterError;
mod node_info;
use crate::node_info::NodeInfo;
mod get_neighbors;
use get_neighbors::Neighbors;
mod options;
use crate::options::Options;
mod render_to_prometheus;
use render_to_prometheus::RenderToPrometheus;

fn handle_request(
    req: Request<Body>,
    options: &Options,
) -> impl Future<Item = Response<Body>, Error = failure::Error> {
    debug!("{:?}", req);

    let r = if options.exclude_neighbors {
        perform_request_simple(req, options)
    } else {
        perform_request_with_neighbors(req, options)
    };

    r.then(|res| match res {
        Ok(body) => ok(body),
        Err(inner_error) => match inner_error {
            ExporterError::UnsupportedPathError { path: ref _path } => {
                warn!("{:?}", inner_error);
                let r = Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(hyper::Body::empty())
                    .unwrap();
                ok(r)
            }
            ExporterError::UnsupportedMethodError { verb: ref _verb } => {
                warn!("{:?}", inner_error);
                let r = Response::builder()
                    .status(StatusCode::METHOD_NOT_ALLOWED)
                    .body(hyper::Body::empty())
                    .unwrap();
                ok(r)
            }
            _ => {
                error!("{:?}", inner_error);
                let r = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(hyper::Body::empty())
                    .unwrap();
                ok(r)
            }
        },
    })
}

#[inline]
fn extract_body(
    req: hyper::client::ResponseFuture,
) -> Box<Future<Item = String, Error = ExporterError> + Send> {
    Box::new(req.from_err().and_then(|resp| {
        debug!("response == {:?}", resp);
        let (_parts, body) = resp.into_parts();
        body.concat2()
            .from_err()
            .and_then(|complete_body| done(String::from_utf8(complete_body.to_vec())).from_err())
    }))
}

fn perform_request_with_neighbors(
    req: Request<Body>,
    options: &Options,
) -> Box<Future<Item = Response<Body>, Error = ExporterError> + Send> {
    debug!("perform_request_with_neighbors");
    Box::new(
        done(check_params_and_prepare_requests(req, options))
            .from_err()
            .and_then(|(future_node_info, future_exclude_neighbors)| {
                extract_body(future_node_info)
                    .join(extract_body(future_exclude_neighbors.unwrap()))
                    .from_err()
                    .and_then(|(node_info_str, neighbors_info_str)| {
                        debug!("node_info_str == {:?}", node_info_str);
                        debug!("neighbors_info_str == {:?}", neighbors_info_str);

                        done(serde_json::from_str(&node_info_str))
                            .from_err()
                            .and_then(move |node_info: NodeInfo| {
                                debug!("{:?}", node_info);

                                done(serde_json::from_str(&neighbors_info_str))
                                    .from_err()
                                    .and_then(move |get_neighbors: Neighbors| {
                                        debug!("{:?}", get_neighbors);
                                        let response = format!(
                                            "{}\n{}",
                                            node_info.render(),
                                            get_neighbors.render()
                                        );
                                        ok(Response::new(Body::from(response)))
                                    })
                            })
                    })
            }),
    )
}

fn perform_request_simple(
    req: Request<Body>,
    options: &Options,
) -> Box<Future<Item = Response<Body>, Error = ExporterError> + Send> {
    debug!("perform_request_simple");
    Box::new(
        done(check_params_and_prepare_requests(req, options))
            .from_err()
            .and_then(|(future_node_info, _)| {
                extract_body(future_node_info)
                    .from_err()
                    .and_then(|node_info_str| {
                        debug!("node_info_str == {:?}", node_info_str);

                        done(serde_json::from_str(&node_info_str))
                            .from_err()
                            .and_then(|node_info: NodeInfo| {
                                debug!("{:?}", node_info);
                                ok(Response::new(Body::from(node_info.render())))
                            })
                    })
            }),
    )
}

fn check_params_and_prepare_requests(
    req: Request<Body>,
    options: &Options,
) -> Result<
    (
        hyper::client::ResponseFuture,
        Option<hyper::client::ResponseFuture>,
    ),
    ExporterError,
> {
    if req.method() != hyper::Method::GET {
        warn!("Only GET is supported, received {}", req.method());
        Err(ExporterError::UnsupportedMethodError {
            verb: req.method().to_string(),
        }
        .into())
    } else if req.uri() != "/metrics" {
        Err(ExporterError::UnsupportedPathError {
            path: req.uri().to_string(),
        }
        .into())
    } else {
        //let uri: hyper::Uri = "http://dns.mindflavor.it:14267".parse().unwrap();
        let cli = Client::new();

        let mut request = hyper::Request::builder();
        request
            .method("PUT")
            .uri(options.iri_uri.clone())
            .header("X-IOTA-API-Version", "1")
            .header("Content-Type", "application/json");
        let request_node_info = request.body(Body::from("{\"command\": \"getNodeInfo\"}"))?;

        if options.exclude_neighbors {
            Ok((cli.request(request_node_info), None))
        } else {
            let mut request = hyper::Request::builder();
            request
                .method("PUT")
                .uri(options.iri_uri.clone())
                .header("X-IOTA-API-Version", "1")
                .header("Content-Type", "application/json");
            let request_neighbors = request.body(Body::from("{\"command\": \"getNeighbors\"}"))?; //O

            Ok((
                cli.request(request_node_info),
                Some(cli.request(request_neighbors)),
            ))
        }
    }
}

fn main() {
    let matches = clap::App::new("prometheus_iota_exporter")
        .version("0.1")
        .author("Francesco Cogno <francesco.cogno@outlook.com>")
        .arg(
            Arg::with_name("iri address")
                .short("a")
                .help("IRI address")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .help("exporter port (default 9978)")
                .default_value("9978")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .help("verbose logging")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("exclude neighbors")
                .short("n")
                .help("do not ask exclude_neighbors")
                .takes_value(false),
        )
        .get_matches();

    let options = Options::from_claps(&matches).unwrap();

    if options.verbose {
        env::set_var("RUST_LOG", "actix_web=trace,prometheus_iota_exporter=trace");
    } else {
        env::set_var("RUST_LOG", "actix_web=info,prometheus_iota_exporter=info");
    }
    env_logger::init();

    info!("using options: {:?}", options);

    let bind = matches.value_of("port").unwrap();
    let bind = u16::from_str_radix(&bind, 10).expect("port must be a valid number");
    let addr = ([0, 0, 0, 0], bind).into();

    info!("starting exporter on {}", addr);

    let new_svc = move || {
        let options = options.clone();
        service_fn(move |req| handle_request(req, &options))
    };

    let server = Server::bind(&addr)
        .serve(new_svc)
        .map_err(|e| eprintln!("server error: {}", e));
    // Run this server for... forever!
    hyper::rt::run(server);
}
