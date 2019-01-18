#[derive(Debug, Clone)]
pub struct Options {
    pub iri_uri: hyper::Uri,
    pub verbose: bool,
    pub exclude_neighbors: bool,
}

impl Options {
    pub fn from_claps(matches: &clap::ArgMatches<'_>) -> Result<Options, failure::Error> {
        let iri_uri = matches.value_of("iri address").unwrap().parse()?;
        let exclude_neighbors = matches.is_present("exclude neighbors");
        let verbose = matches.is_present("verbose");

        Ok(Options {
            iri_uri,
            verbose,
            exclude_neighbors,
        })
    }
}
