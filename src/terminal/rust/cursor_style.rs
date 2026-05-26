#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CursorVisualStyle {
    Bar,
    #[default]
    Block,
    Underline,
    BlockHollow,
}
