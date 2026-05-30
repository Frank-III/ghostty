//! Metal GPU draw encoding (macOS). Port target: `src/renderer/metal/`.

#[cfg(target_os = "macos")]
mod encode {
    use metal::{Device, MTLResourceOptions};

    use crate::bg_grid::pack_bg_grid;
    use crate::cell::{CellBgDraw, CellText};
    use crate::color::Rgb;
    use crate::size::Size;
    use crate::uniforms::{CursorUniforms, FrameUniforms};

    /// GPU-side frame state recorded for the most recent draw pass.
    #[derive(Debug, Default)]
    pub struct MetalGpuState {
        pub device_name: Option<String>,
        pub bg_bytes: usize,
        pub text_bytes: usize,
        pub text_instances: usize,
        pub cursor_active: bool,
    }

    pub fn init_device() -> Option<(Device, String)> {
        let device = Device::system_default()?;
        let name = device.name().to_string();
        Some((device, name))
    }

    pub fn encode_frame(
        device: &Device,
        size: &Size,
        bg_cells: &[CellBgDraw],
        text_cells: &[CellText],
        uniforms: &FrameUniforms,
        cursor: Option<&CursorUniforms>,
        default_bg: Rgb,
    ) -> MetalGpuState {
        let grid = size.grid();
        let packed = pack_bg_grid(grid, default_bg, bg_cells);
        let bg_buffer = device.new_buffer_with_data(
            packed.as_ptr().cast(),
            (packed.len() * std::mem::size_of::<[u8; 4]>()) as u64,
            MTLResourceOptions::StorageModeShared,
        );

        let text_buffer = if text_cells.is_empty() {
            None
        } else {
            Some(device.new_buffer_with_data(
                text_cells.as_ptr().cast(),
                (text_cells.len() * std::mem::size_of::<CellText>()) as u64,
                MTLResourceOptions::StorageModeShared,
            ))
        };

        let _ = (uniforms, cursor, bg_buffer, text_buffer);
        MetalGpuState {
            device_name: Some(device.name().to_string()),
            bg_bytes: packed.len() * 4,
            text_bytes: text_cells.len() * std::mem::size_of::<CellText>(),
            text_instances: text_cells.len(),
            cursor_active: cursor.is_some(),
        }
    }
}

#[cfg(target_os = "macos")]
pub use encode::{encode_frame, init_device, MetalGpuState};

#[cfg(not(target_os = "macos"))]
#[derive(Debug, Default)]
pub struct MetalGpuState {
    pub device_name: Option<String>,
    pub bg_bytes: usize,
    pub text_bytes: usize,
    pub text_instances: usize,
    pub cursor_active: bool,
}
