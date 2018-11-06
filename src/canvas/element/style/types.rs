#[derive(Clone, Copy, PartialEq)]
pub enum DisplayType {
    None,
    Block,
    Inline,
    InlineBlock,
    Flex,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PositionType {
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}
