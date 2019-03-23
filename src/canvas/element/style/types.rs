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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BoxSizingType {
    Default,
    ContentBox,
    PaddingBox,
    BorderBox,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BorderStyleType {
    None,
    Solid,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FlexDirectionType {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}
