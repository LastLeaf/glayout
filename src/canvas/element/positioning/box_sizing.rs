use super::super::{ElementStyle, BoxSizingType, PositionType, DEFAULT_F64};
use super::{Point, Size};

#[inline]
pub fn get_sizes(style: &ElementStyle, default_size: Size) -> (Size, Size, Size, Size) {
    let style_content_width = style.get_width();
    let style_content_height = style.get_height();
    let style_padding_width = style.get_padding_left() + style.get_padding_right();
    let style_padding_height = style.get_padding_top() + style.get_padding_bottom();
    let style_border_width = style.get_border_left_width() + style.get_border_right_width();
    let style_border_height = style.get_border_top_width() + style.get_border_bottom_width();
    let style_margin_width = style.get_margin_left() + style.get_margin_right();
    let style_margin_height = style.get_margin_top() + style.get_margin_bottom();
    let content = Size::new(
        if style_content_width == DEFAULT_F64 {
            match style.get_box_sizing() {
                BoxSizingType::Default => default_size.width(),
                BoxSizingType::ContentBox => default_size.width(),
                BoxSizingType::PaddingBox => default_size.width() - style_padding_width,
                BoxSizingType::BorderBox => default_size.width() - style_padding_width - style_border_width,
            }
        } else { style_content_width },
        if style_content_height == DEFAULT_F64 {
            default_size.height()
        } else { style_content_height },
    );
    let padding = Size::new(
        content.width() + style_padding_width,
        content.height() + style_padding_height,
    );
    let border = Size::new(
        content.width() + style_border_width,
        content.height() + style_border_height,
    );
    let margin = Size::new(
        border.width() + style_margin_width,
        border.height() + style_margin_height,
    );
    (margin, border, padding, content)
}

#[inline]
pub fn get_offsets(style: &ElementStyle) -> (Point, Point, Point, Point) {
    let margin = Point::new(0., 0.);
    let border = Point::new(
        margin.left() + style.get_margin_left(),
        margin.top() + style.get_margin_top(),
    );
    let padding = Point::new(
        border.left() + style.get_border_left_width(),
        border.top() + style.get_border_top_width(),
    );
    let content = Point::new(
        padding.left() + style.get_padding_left(),
        padding.top() + style.get_padding_top(),
    );
    (margin, border, padding, content)
}

#[inline]
pub fn is_independent_positioning(style: &ElementStyle) -> bool {
    let position = style.get_position();
    position == PositionType::Absolute || position == PositionType::Fixed
}
