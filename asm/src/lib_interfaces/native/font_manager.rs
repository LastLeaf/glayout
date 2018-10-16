use std::os::raw::c_char;
use std::ffi::CStr;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use euclid::{Point2D, Size2D};
use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::family_name::FamilyName;
use font_kit::hinting::HintingOptions;
use font_kit::properties::{Properties, Style, Weight};
use font_kit::source::SystemSource;
use font_kit::metrics::Metrics;
use font_kit::font::Font;

lazy_static! {
	static ref DEFAULT_FONT_FAMILY_ID: i32 = init_default_font_family();
	static ref FONT_FAMILIE_NAMES: Arc<Mutex<HashMap<i32, String>>> = Arc::new(Mutex::new(HashMap::new()));
	static ref FONT_INFO: Arc<Mutex<HashMap<FontInfoKey, Vec<SingleFontFamily>>>> = Arc::new(Mutex::new(HashMap::new()));
	static ref CURRENT_FONT: Arc<Mutex<FontSettings>> = Arc::new(Mutex::new(FontSettings::new()));
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct FontInfoKey {
	font_family_id: i32,
	italic: i32,
	bold: i32,
}

struct SingleFontFamily {
	font: Font,
	metrics: Metrics,
}

#[inline]
fn get_glyph_size(font: &Font, font_metrics: &Metrics, glyph_id: u32, font_size: f32) -> (f32, f32) {
	let v = font.advance(glyph_id).unwrap();
	let scale = font_size / font_metrics.units_per_em as f32;
	(v.x * scale, v.y * scale)
}

#[inline]
fn select_font(fonts_info: &Vec<SingleFontFamily>, glyph: char) -> (&SingleFontFamily, u32) {
	let mut glyph_id = 0;
	let mut font_family = None;
	for f in fonts_info.iter() {
		font_family = Some(f);
		glyph_id = f.font.glyph_for_char(glyph).unwrap();
		if glyph_id != 0 {
			return (f, glyph_id);
		}
	}
	(font_family.unwrap(), glyph_id)
}

fn load_font_family(names: &String, properties: &Properties) -> Vec<SingleFontFamily> {
	names.split(',').map(|s| {
		let name = String::from(s.trim());
		let name = if (name.starts_with('"') && name.ends_with('"')) || (name.starts_with('\'') && name.ends_with('\'')) {
			(name[1..name.len() - 1]).to_string()
		} else {
			name
		};
		let family_name = match name.as_str() {
			"serif" => {
				FamilyName::Serif
			},
			"sans-serif" => {
				FamilyName::SansSerif
			},
			"monospace" => {
				FamilyName::Monospace
			},
			"cursive" => {
				FamilyName::Cursive
			},
			"fantasy" => {
				FamilyName::Fantasy
			},
			_ => {
				FamilyName::Title(name)
			}
		};
		let font = SystemSource::new()
			.select_best_match(&[family_name], properties)
		    .unwrap()
		    .load()
		    .unwrap();
		let metrics = font.metrics();
		SingleFontFamily {
			font,
			metrics,
		}
	}).collect()
}

fn init_default_font_family() -> i32 {
	let key = FontInfoKey {
		font_family_id: -1,
		italic: 0,
		bold: 0,
	};
	FONT_INFO.lock().unwrap().insert(key, load_font_family(&String::from("sans-serif"), &Properties::new()));
	-1
}

pub fn init() {
	assert!(*DEFAULT_FONT_FAMILY_ID == -1);
}

struct FontSettings {
	font_size: i32,
	line_height: i32,
	font_info: FontInfoKey,
}

impl FontSettings {
	fn new() -> Self {
		Self {
			font_size: 16,
			line_height: 24,
			font_info: FontInfoKey {
				font_family_id: -1,
				italic: 0,
				bold: 0,
			},
		}
	}
}

pub fn text_bind_font_family(id: i32, font_family: *mut c_char) {
	let n = unsafe { CStr::from_ptr(font_family as *const i8).to_string_lossy().into_owned() };
    FONT_FAMILIE_NAMES.lock().unwrap().insert(id, n);
}
pub fn text_unbind_font_family(id: i32) {
    FONT_FAMILIE_NAMES.lock().unwrap().remove(&id);
}
pub fn text_set_font(font_size: i32, line_height: i32, font_family_id: i32, italic: i32, bold: i32) {
    let mut current_font = CURRENT_FONT.lock().unwrap();
	current_font.font_size = font_size;
	current_font.line_height = line_height;
	current_font.font_info.font_family_id = font_family_id;
	current_font.font_info.italic = italic;
	current_font.font_info.bold = bold;
}
pub fn text_get_width(text: *mut c_char) -> f64 {
	let current_font = CURRENT_FONT.lock().unwrap();
	let mut font_info = FONT_INFO.lock().unwrap();
	if font_info.get(&current_font.font_info).is_none() {
		let mut properties = Properties::new();
		match current_font.font_info.italic {
			1 => { properties.style = Style::Italic },
			_ => { }
		};
		match current_font.font_info.bold {
			1 => { properties.weight = Weight::BOLD },
			_ => { }
		};
		let fonts = load_font_family(FONT_FAMILIE_NAMES.lock().unwrap().get(&current_font.font_info.font_family_id).unwrap(), &properties);
		font_info.insert(current_font.font_info.clone(), fonts);
	}
	let fonts: &Vec<SingleFontFamily> = &font_info[&current_font.font_info];
	let s = unsafe { CStr::from_ptr(text as *const i8).to_str().unwrap() };
	let mut total_width = 0.;
	for c in s.chars() {
    	let (single_font_family, glyph_id) = select_font(fonts, c);
	  	total_width += get_glyph_size(&single_font_family.font, &single_font_family.metrics, glyph_id, current_font.font_size as f32).0;
	}
	total_width as f64
}
pub fn text_to_tex(canvas_index: i32, tex_id: i32, tex_left: i32, tex_top: i32, text: *mut c_char, width: i32, height: i32, line_height: i32) {
	let current_font = CURRENT_FONT.lock().unwrap();
	let font_info = FONT_INFO.lock().unwrap();
    let mut canvas = Canvas::new(&Size2D::new(width as u32, height as u32), Format::A8);
	let fonts: &Vec<SingleFontFamily> = &font_info[&current_font.font_info];
	let s = unsafe { CStr::from_ptr(text as *const i8).to_str().unwrap() };
	let mut offset_x = 0.;
	let mut offset_y = (line_height - current_font.font_size) as f32 / 2.;
	for c in s.chars() {
		if c == '\n' {
			offset_x = 0.;
			offset_y += line_height as f32;
		} else if c >= ' ' {
			let (single_font_family, glyph_id) = select_font(fonts, c);
			let (w, _) = get_glyph_size(&single_font_family.font, &single_font_family.metrics, glyph_id, current_font.font_size as f32);
			if c != ' ' {
				let pos_x = offset_x * single_font_family.metrics.units_per_em as f32 / current_font.font_size as f32;
				let pos_y = - offset_y * single_font_family.metrics.units_per_em as f32 / current_font.font_size as f32;
				single_font_family.font.rasterize_glyph(&mut canvas, glyph_id, current_font.font_size as f32, &Point2D::new(pos_x, pos_y), HintingOptions::None, RasterizationOptions::GrayscaleAa).unwrap();
			}
			offset_x += w;
		}
	}
	let mut buf: Vec<u8> = Vec::with_capacity(canvas.pixels.len() * 4);
	for a in canvas.pixels.into_iter() {
		buf.extend_from_slice(&[0, 0, 0, a]);
	}
    super::tex_manager::tex_rewrite(canvas_index, buf, tex_id, tex_left, tex_top, width, height);
}
