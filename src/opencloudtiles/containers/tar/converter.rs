use crate::opencloudtiles::{
	containers::abstract_container,
	progress::ProgressBar,
	types::{TileConverterConfig, TileFormat},
};
use std::{fs::File, path::Path};
use tar::{Builder, Header};

pub struct TileConverter {
	builder: Builder<File>,
	config: TileConverterConfig,
}
impl abstract_container::TileConverter for TileConverter {
	fn new(
		filename: &std::path::PathBuf, config: TileConverterConfig,
	) -> Box<dyn abstract_container::TileConverter>
	where
		Self: Sized,
	{
		let file = File::create(filename).unwrap();
		let builder = Builder::new(file);

		Box::new(TileConverter { builder, config })
	}
	fn convert_from(&mut self, reader: Box<dyn abstract_container::TileReader>) {
		self.config.finalize_with_parameters(reader.get_parameters());

		let converter = self.config.get_tile_converter();

		let ext = match self.config.get_tile_format() {
			TileFormat::PBF => "pbf",
			TileFormat::PBFGzip => "pbf.gz",
			TileFormat::PBFBrotli => "pbf.br",
			TileFormat::PNG => "png",
			TileFormat::JPG => "jpg",
			TileFormat::WEBP => "webp",
		};

		let bbox_pyramide = self.config.get_bbox_pyramide();
		let mut bar = ProgressBar::new("counting tiles", bbox_pyramide.count_tiles());

		for coord in bbox_pyramide.iter_tile_indexes() {
			bar.inc(1);

			let tile = reader.get_tile_data(&coord);
			if tile.is_none() {
				continue;
			}

			let tile_data = tile.unwrap();
			let tile_compressed = converter(&tile_data);

			//println!("{}", &tile_data.len());

			let filename = format!("./{}/{}/{}.{}", coord.z, coord.y, coord.x, ext);
			let path = Path::new(&filename);
			let mut header = Header::new_gnu();
			header.set_size(tile_compressed.len() as u64);
			header.set_mode(0o644);

			self
				.builder
				.append_data(&mut header, &path, tile_compressed.as_slice())
				.unwrap();
		}
		bar.finish();
		self.builder.finish().unwrap();
	}
}