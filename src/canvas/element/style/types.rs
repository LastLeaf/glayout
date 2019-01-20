#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DisplayType {
    None,
    Block,
    Inline,
    InlineBlock,
    Flex,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PositionType {
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextAlignType {
    Left,
    Center,
    Right,
}
