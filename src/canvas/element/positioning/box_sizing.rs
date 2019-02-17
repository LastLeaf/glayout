use super::super::{ElementStyle, BoxSizingType, PositionType, DEFAULT_F64};
use super::{Point, Size};

#[inline]
pub fn get_sizes(style: &ElementStyle, outer_size: Size) -> (Size, Size, Size, Size) {
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
            if outer_size.width() == DEFAULT_F64 {
                0.
            } else {
                outer_size.width() - style_padding_width - style_border_width - style_margin_width
            }
        } else {
            match style.get_box_sizing() {
                BoxSizingType::Default => style_content_width,
                BoxSizingType::ContentBox => style_content_width,
                BoxSizingType::PaddingBox => style_content_width - style_padding_width,
                BoxSizingType::BorderBox => style_content_width - style_padding_width - style_border_width,
            }
        },
        if style_content_height == DEFAULT_F64 {
            if outer_size.height() == DEFAULT_F64 {
                0.
            } else {
                outer_size.height() - style_padding_height - style_border_height - style_margin_height
            }
        } else {
            match style.get_box_sizing() {
                BoxSizingType::Default => style_content_height,
                BoxSizingType::ContentBox => style_content_height,
                BoxSizingType::PaddingBox => style_content_height - style_padding_height,
                BoxSizingType::BorderBox => style_content_height - style_padding_height - style_border_height,
            }
        },
    );
    let padding = Size::new(
        content.width() + style_padding_width,
        content.height() + style_padding_height,
    );
    let border = Size::new(
        padding.width() + style_border_width,
        padding.height() + style_border_height,
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
