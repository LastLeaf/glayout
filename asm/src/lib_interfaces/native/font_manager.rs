use euclid::{Point2D, Size2D};
use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::family_name::FamilyName;
use font_kit::hinting::HintingOptions;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;

pub fn init() {
	let font = SystemSource::new()
		.select_best_match(&[FamilyName::Title(String::from("宋体"))], &Properties::new())
	    .unwrap()
	    .load()
	    .unwrap();
	let glyph_id = font.glyph_for_char('字').unwrap();
	let mut canvas = Canvas::new(&Size2D::new(32, 32), Format::A8);
	let bounds = font.raster_bounds(glyph_id, 32.0, &Point2D::zero(), HintingOptions::None, RasterizationOptions::GrayscaleAa).unwrap();
	font.rasterize_glyph(&mut canvas, glyph_id, 32.0, &Point2D::zero(), HintingOptions::None, RasterizationOptions::GrayscaleAa).unwrap();
	println!("{:?}, {:?}, {:?}", glyph_id, bounds, canvas.pixels);
}
