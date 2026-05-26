#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyMousePosition {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct GhosttyMouseSize {
    pub(crate) screen_width: u32,
    pub(crate) screen_height: u32,
    pub(crate) cell_width: u32,
    pub(crate) cell_height: u32,
    pub(crate) padding_top: u32,
    pub(crate) padding_bottom: u32,
    pub(crate) padding_right: u32,
    pub(crate) padding_left: u32,
}

#[derive(Clone, Copy)]
pub(crate) struct GhosttyMouseCell {
    pub(crate) x: u16,
    pub(crate) y: u32,
}

#[derive(Clone, Copy)]
pub(crate) struct GhosttyMousePixels {
    pub(crate) x: i32,
    pub(crate) y: i32,
}
