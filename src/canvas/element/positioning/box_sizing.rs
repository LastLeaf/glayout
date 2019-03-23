use super::super::{ElementStyle, BoxSizingType, PositionType, StyleValueReferrer};
use super::{Point, Size};

#[inline]
pub(super) fn get_sizes(style: &ElementStyle, outer_size: Size) -> (Size, Size, Size, Size) {
    let style_content_width = style.get_width();
    let style_content_height = style.get_height();
    let style_padding_width = style.get_padding_left() + style.get_padding_right();
    let style_padding_height = style.get_padding_top() + style.get_padding_bottom();
    let style_border_width = style.get_border_left_width() + style.get_border_right_width();
    let style_border_height = style.get_border_top_width() + style.get_border_bottom_width();
    let style_margin_width = style.get_margin_left() + style.get_margin_right();
    let style_margin_height = style.get_margin_top() + style.get_margin_bottom();
    let content = Size::new(
        if !style_content_width.is_finite() {
            if !outer_size.width().is_finite() {
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
        if !style_content_height.is_finite() {
            if !outer_size.height().is_finite() {
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
pub(super) fn get_offsets(style: &ElementStyle, suggested_size: Size, requested_size: Size) -> (Point, Point, Point, Point) {
    let margin = Point::new(0., 0.);
    let border = Point::new(
        if style.get_margin_left_advanced().0 != StyleValueReferrer::Auto {
            margin.left() + style.get_margin_left()
        } else if style.get_margin_right_advanced().0 != StyleValueReferrer::Auto {
            margin.left()
        } else {
            let r = (suggested_size.width() - requested_size.width()) / 2.;
            if r > 0. { r } else { 0. }
        },
        if style.get_margin_top_advanced().0 != StyleValueReferrer::Auto {
            margin.top() + style.get_margin_top()
        } else if style.get_margin_bottom_advanced().0 != StyleValueReferrer::Auto {
            margin.top()
        } else {
            let r = (suggested_size.height() - requested_size.height()) / 2.;
            if r > 0. { r } else { 0. }
        },
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
pub(super) fn is_independent_positioning(style: &ElementStyle) -> bool {
    let position = style.get_position();
    position == PositionType::Absolute || position == PositionType::Fixed
}
