#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;
use clap::{crate_authors, crate_name, crate_version, Arg};
use futures::future::{done, ok, Either, Future};
use futures::stream::Stream;
use hyper::Client;
use hyper::{Body, Request, Response};
use log::{debug, info, trace};
use std::env;
mod exporter_error;
use crate::exporter_error::ExporterError;
mod node_info;
use crate::node_info::NodeInfo;
use std::sync::Arc;
mod get_neighbors;
use get_neighbors::Neighbors;
mod options;
use crate::options::Options;
use serde::de::DeserializeOwned;
mod render_to_prometheus;
use prometheus_exporter_base::render_prometheus;
use render_to_prometheus::RenderToPrometheus;

#[inline]
fn extract_body(
    req: hyper::client::ResponseFuture,
) -> impl Future<Item = String, Error = ExporterError> + Send {
    req.from_err().and_then(|resp| {
        debug!("response == {:?}", resp);
        let (_parts, body) = resp.into_parts();
        body.concat2()
            .from_err()
            .and_then(|complete_body| done(String::from_utf8(complete_body.to_vec())).from_err())
    })
}

fn perform_request(
    _req: Request<Body>,
    options: &Arc<Options>,
) -> impl Future<Item = Response<Body>, Error = failure::Error> {
    trace!("perform_request");

    let fut_get_node_info = create_iri_future::<NodeInfo>("getNodeInfo", options);

    if options.exclude_neighbors {
        // in this case we only query the getNodeInfo method
        Either::A(
            fut_get_node_info
                .and_then(|node_info| ok(Response::new(Body::from(node_info.render())))),
        )
    } else {
        Either::B({
            // we add the getNeighbors method
            let fut_get_neighbors = create_iri_future::<Neighbors>("getNeighbors", options);

            // we join the two futures so they will run concurrently
            fut_get_node_info
                .join(fut_get_neighbors)
                .and_then(|(node_info, get_neighbors)| {
                    let response = format!("{}\n{}", node_info.render(), get_neighbors.render());
                    ok(Response::new(Body::from(response)))
                })
        })
    }
}

fn create_iri_future<T>(
    command: &str,
    options: &Arc<Options>,
) -> impl Future<Item = T, Error = failure::Error>
where
    T: DeserializeOwned + std::fmt::Debug,
{
    let cli = Client::new();
    let mut request = hyper::Request::builder();

    done(
        request
            .method("PUT")
            .uri(options.iri_uri.clone())
            .header("X-IOTA-API-Version", "1")
            .header("Content-Type", "application/json")
            .body(Body::from(format!("{{\"command\": \"{}\"}}", command))),
    )
    .from_err()
    .and_then(move |request| extract_body(cli.request(request)).from_err())
    .and_then(|text: String| {
        debug!("received_text == {:?}", text);
        ok(text)
    })
    .and_then(|text| done(serde_json::from_str(&text)).from_err())
    .and_then(|t: T| {
        debug!("received_object == {:?}", t);
        ok(t)
    })
}

fn main() {
    let matches = clap::App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
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
                .help("exporter port")
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
                .help("do not include getNeighbors method results")
                .takes_value(false),
        )
        .get_matches();

    let options = Options::from_claps(&matches).unwrap();

    if options.verbose {
        env::set_var(
            "RUST_LOG",
            format!("{}=trace,prometheus_exporter_base=trace", crate_name!()),
        );
    } else {
        env::set_var(
            "RUST_LOG",
            format!("{}=info,prometheus_exporter_base=info", crate_name!()),
        );
    }
    env_logger::init();

    info!("using options: {:?}", options);

    let bind = matches.value_of("port").unwrap();
    let bind = u16::from_str_radix(&bind, 10).expect("port must be a valid number");
    let addr = ([0, 0, 0, 0], bind).into();

    info!("starting exporter on {}", addr);

    render_prometheus(&addr, options, |request, options| {
        Box::new(perform_request(request, options))
    });
}
