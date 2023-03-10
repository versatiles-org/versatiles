use crate::{
	server::{source, TileServer},
	tools::get_reader,
};
use clap::Args;
use regex::Regex;

#[derive(Args)]
#[command(arg_required_else_help = true, disable_version_flag = true, verbatim_doc_comment)]
pub struct Subcommand {
	/// One or more tile containers you want to serve.
	/// Supported container formats are: *.versatiles, *.tar, *.mbtiles
	/// Container files have to be on the local filesystem, except VersaTiles containers:
	///    VersaTiles containers can also be served from http://..., https://... or gs://...
	/// The name used in the url (/tiles/$name/) will be generated automatically from the file name:
	///    e.g. ".../ukraine.versatiles" will be served at url "/tiles/ukraine/..."
	/// You can also configure a different name for each file using:
	///    "[name]file", "file[name]" or "file#name"
	#[arg(num_args = 1.., required = true, verbatim_doc_comment)]
	pub sources: Vec<String>,

	/// Serve via socket ip.
	#[arg(short = 'i', long, default_value = "127.0.0.1")]
	pub ip: String,

	/// Serve via port.
	#[arg(short, long, default_value = "8080")]
	pub port: u16,

	/// Serve static content at "http:/.../" from a local folder or tar.
	/// If multiple static sources are defined, the first hit will be served.
	#[arg(short = 's', long = "static")]
	pub static_content: Vec<String>,
}

pub fn run(arguments: &Subcommand) {
	let mut server: TileServer = new_server(arguments);

	let patterns: Vec<Regex> = [
		r"^\[(?P<name>[a-z0-9-]+?)\](?P<url>.*)$",
		r"^(?P<url>.*)\[(?P<name>[a-z0-9-]+?)\]$",
		r"^(?P<url>.*)#(?P<name>[a-z0-9-]+?)$",
		r"^(?P<url>.*)$",
	]
	.iter()
	.map(|pat| Regex::new(pat).unwrap())
	.collect();

	arguments.sources.iter().for_each(|arg| {
		let pattern = patterns.iter().find(|p| p.is_match(arg)).unwrap();
		let c = pattern.captures(arg).unwrap();

		let url: &str = c.name("url").unwrap().as_str();
		let name: &str = match c.name("name") {
			None => {
				let filename = url.split(&['/', '\\']).last().unwrap();
				filename.split('.').next().unwrap()
			}
			Some(m) => m.as_str(),
		};

		let reader = get_reader(url);
		server.add_source(format!("/tiles/{name}/"), source::TileContainer::from(reader));
	});

	for filename in arguments.static_content.iter() {
		if filename.ends_with(".tar") {
			server.add_static(source::TarFile::from(filename));
		} else {
			server.add_static(source::Folder::from(filename));
		}
	}

	let mut list: Vec<(String, String)> = server.iter_url_mapping().collect();
	list.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
	list
		.iter()
		.for_each(|(url, source)| println!("   {:30}  <-  {}", url.to_owned() + "*", source));

	server.start();
}

fn new_server(command: &Subcommand) -> TileServer {
	TileServer::new(&command.ip, command.port)
}
