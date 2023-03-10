use crate::tools::get_reader;
use clap::Args;

#[derive(Args)]
#[command(arg_required_else_help = true, disable_version_flag = true)]
pub struct Subcommand {
	/// tile container you want to probe
	/// supported container formats are: *.versatiles, *.tar, *.mbtiles
	#[arg(required = true, verbatim_doc_comment)]
	file: String,

	/// deep scan of every tile
	#[arg(long, short)]
	deep: bool,
}

pub fn run(arguments: &Subcommand) {
	println!("probe {:?}", arguments.file);

	let reader = get_reader(&arguments.file);
	println!("{reader:#?}");

	if arguments.deep {
		reader.deep_verify();
	}
}
