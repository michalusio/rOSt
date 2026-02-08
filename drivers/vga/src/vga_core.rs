use lazy_static::lazy_static;
use noto_sans_mono_bitmap::{
    FontWeight, RasterHeight, RasterizedChar, get_raster, get_raster_width,
};

pub const CHAR_HEIGHT: RasterHeight = RasterHeight::Size16;
pub const CHAR_WEIGHT: FontWeight = FontWeight::Regular;
pub const CHAR_WIDTH: u16 = get_raster_width(CHAR_WEIGHT, CHAR_HEIGHT) as u16;
lazy_static! {
    pub static ref INVALID_CHAR: RasterizedChar =
        get_raster(' ', CHAR_WEIGHT, CHAR_HEIGHT).unwrap();
}
