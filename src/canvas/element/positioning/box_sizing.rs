use super::super::{Element, ElementStyle, BoxSizingType, PositionType, StyleValueReferrer};
use super::{Point, Size};

#[inline]
pub(super) fn get_h_offset(style: &ElementStyle) -> (f64, bool) {
    let mut spec_width = true;

    let style_padding_width = style.get_padding_left() + style.get_padding_right();
    let style_border_width = style.get_border_left_width() + style.get_border_right_width();
    let style_margin_width = style.get_margin_left() + style.get_margin_right();

    let style_content_width = style.get_width();
    let offset = if !style_content_width.is_finite() {
        spec_width = false;
        style_padding_width + style_border_width + style_margin_width
    } else {
        match style.get_box_sizing() {
            BoxSizingType::Default | BoxSizingType::ContentBox => style_content_width + style_padding_width + style_border_width + style_margin_width,
            BoxSizingType::PaddingBox => style_content_width + style_border_width + style_margin_width,
            BoxSizingType::BorderBox => style_content_width + style_margin_width,
        }
    };

    (offset, spec_width)
}

#[inline]
pub(super) fn get_sizes(element: &mut Element, style: &ElementStyle, outer_size: Size) -> (Size, Size, Size, Size, bool, bool) {
    let mut spec_width = true;
    let mut spec_height = true;

    let style_padding_width = style.get_padding_left() + style.get_padding_right();
    let style_padding_height = style.get_padding_top() + style.get_padding_bottom();
    let style_border_width = style.get_border_left_width() + style.get_border_right_width();
    let style_border_height = style.get_border_top_width() + style.get_border_bottom_width();
    let style_margin_width = style.get_margin_left() + style.get_margin_right();
    let style_margin_height = style.get_margin_top() + style.get_margin_bottom();

    let content = if is_independent_positioning(style) {
        let width = if style.get_width().is_finite() {
            style.get_width()
        } else {
            let mut max_width = outer_size.width();
            if style.get_left().is_finite() {
                max_width -= style.get_left();
            }
            if style.get_right().is_finite() {
                max_width -= style.get_right();
            }
            if style.get_left().is_finite() && style.get_right().is_finite() {
                max_width
            } else {
                let (_, max) = element.position_offset.min_max_width();
                if max < max_width { max } else { max_width }
            }
        };
        let height = if style.get_height().is_finite() {
            style.get_height()
        } else {
            if style.get_top().is_finite() && style.get_bottom().is_finite() {
                let mut max_height = outer_size.height();
                max_height -= style.get_top();
                max_height -= style.get_bottom();
                max_height
            } else {
                spec_height = false;
                0.
            }
        };
        Size::new(
            width - style_border_width - style_padding_width,
            height - style_border_height - style_padding_height,
        )
    } else {
        let style_content_width = style.get_width();
        let style_content_height = style.get_height();
        Size::new(
            if !style_content_width.is_finite() {
                if !outer_size.width().is_finite() {
                    spec_width = false;
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
                    spec_height = false;
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
        )
    };

    let content = Size::new(
        if content.width() < 0. { 0. } else { content.width() },
        if content.height() < 0. { 0. } else { content.height() },
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
    (margin, border, padding, content, spec_width, spec_height)
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
            let abs_offset = if is_independent_positioning(style) {
                let mut r = 0.;
                if style.get_left().is_finite() {
                    r += style.get_left();
                }
                if style.get_right().is_finite() {
                    r += style.get_right();
                }
                r
            } else {
                0.
            };
            let r = (suggested_size.width() - requested_size.width() - abs_offset) / 2.;
            if r > 0. { r } else { 0. }
        },
        if style.get_margin_top_advanced().0 != StyleValueReferrer::Auto {
            margin.top() + style.get_margin_top()
        } else if style.get_margin_bottom_advanced().0 != StyleValueReferrer::Auto {
            margin.top()
        } else {
            let abs_offset = if is_independent_positioning(style) {
                let mut r = 0.;
                if style.get_top().is_finite() {
                    r += style.get_top();
                }
                if style.get_bottom().is_finite() {
                    r += style.get_bottom();
                }
                r
            } else {
                0.
            };
            let r = (suggested_size.height() - requested_size.height() - abs_offset) / 2.;
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
