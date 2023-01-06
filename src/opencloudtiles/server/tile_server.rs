use crate::opencloudtiles::lib::Precompression;

use super::traits::ServerSourceBox;
use enumset::{enum_set, EnumSet};
use hyper::{
	header,
	service::{make_service_fn, service_fn},
	Body, Request, Response, Result, Server, StatusCode,
};
use std::{net::SocketAddr, sync::Arc};

type GenericError = Box<dyn std::error::Error + Send + Sync>;

pub struct TileServer {
	port: u16,
	sources: Vec<(String, ServerSourceBox)>,
}

impl TileServer {
	pub fn new(port: u16) -> TileServer {
		return TileServer {
			port,
			sources: Vec::new(),
		};
	}

	pub fn add_source(&mut self, url_prefix: String, source: ServerSourceBox) {
		let mut prefix = url_prefix;
		if !prefix.starts_with("/") {
			prefix = "/".to_owned() + &prefix;
		}
		if !prefix.ends_with("/") {
			prefix = prefix + "/";
		}

		for (other_prefix, _source) in self.sources.iter() {
			if other_prefix.starts_with(&prefix) || prefix.starts_with(other_prefix) {
				panic!(
					"multiple sources with the prefix '{}' and '{}' are defined",
					prefix, other_prefix
				);
			};
		}

		self.sources.push((prefix, source));
	}

	#[tokio::main]
	pub async fn start(&mut self) {
		let addr = SocketAddr::from(([127, 0, 0, 1], self.port));

		let mut sources: Vec<(String, usize, Arc<ServerSourceBox>)> = Vec::new();
		while self.sources.len() > 0 {
			let (prefix, source) = self.sources.pop().unwrap();
			let skip = prefix.matches("/").count();
			sources.push((prefix, skip, Arc::new(source)));
		}
		let arc_sources = Arc::new(sources);

		let new_service = make_service_fn(move |_| {
			let arc_sources = arc_sources.clone();
			async move {
				Ok::<_, GenericError>(service_fn(move |req: Request<Body>| {
					let arc_sources = arc_sources.clone();
					async move {
						let _method = req.method();
						let path = req.uri().path();
						let headers = req.headers();

						let mut encoding_set: EnumSet<Precompression> =
							enum_set!(Precompression::Uncompressed);
						let encoding = headers.get(header::ACCEPT_ENCODING);
						if encoding.is_some() {
							let encoding_string = encoding.unwrap().to_str().unwrap_or("");

							if encoding_string.contains("gzip") {
								encoding_set.insert(Precompression::Gzip);
							}
							if encoding_string.contains("br") {
								encoding_set.insert(Precompression::Brotli);
							}
						}

						let source_option = arc_sources
							.iter()
							.find(|(prefix, _, _)| path.starts_with(prefix));

						if source_option.is_none() {
							return ok_not_found();
						}

						let (_prefix, skip, source) = source_option.unwrap();

						let split_path: Vec<&str> = path.split("/").collect();
						let sub_path: &[&str] = if skip < &split_path.len() {
							&split_path.as_slice()[*skip..]
						} else {
							&[]
						};

						//println!("   - - - ");
						//println!("path {}", path);
						//println!("prefix {}", prefix);
						//println!("skip {}", skip);
						//println!("split_path {:?}", split_path);
						//println!("sub_path {:?}", sub_path);

						let result = source.get_data(sub_path, encoding_set);

						if result.is_err() {
							return ok_not_found();
						}

						return result;
					}
				}))
			}
		});
		let server = Server::bind(&addr).serve(new_service);

		if let Err(e) = server.await {
			eprintln!("server error: {}", e);
		}
	}

	pub fn iter_url_mapping(&self) -> impl Iterator<Item = (String, String)> + '_ {
		self
			.sources
			.iter()
			.map(|(url, source)| (url.to_owned(), source.get_name().to_owned()))
	}
}

pub fn ok_not_found() -> Result<Response<Body>> {
	return Ok(Response::builder()
		.status(StatusCode::NOT_FOUND)
		.body("Not Found".into())
		.unwrap());
}