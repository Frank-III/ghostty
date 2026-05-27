use core::ffi::c_void;
use core::ptr;

use crate::kitty_graphics_command::*;
use crate::kitty_graphics_image::*;
use crate::kitty_graphics_storage::*;

static MSG_OK: &[u8] = b"OK";
static MSG_EINVAL_IMAGE_ID_REQUIRED: &[u8] = b"EINVAL: image ID required";
static MSG_EINVAL_ID_AND_NUMBER_EXCLUSIVE: &[u8] = b"EINVAL: image ID and number are mutually exclusive";
static MSG_EINVAL_ID_OR_NUMBER_REQUIRED: &[u8] = b"EINVAL: image ID or number required";
static MSG_ENOENT_IMAGE_NOT_FOUND: &[u8] = b"ENOENT: image not found";
static MSG_EINVAL_VIRTUAL_PARENT: &[u8] = b"EINVAL: virtual placement cannot refer to a parent";
static MSG_EINVAL_TERMINAL_STATE: &[u8] = b"EINVAL: failed to prepare terminal state";
static MSG_EINVAL_UNIMPLEMENTED: &[u8] = b"ERROR: unimplemented action";
static MSG_ENOMEM: &[u8] = b"ENOMEM: out of memory";
static MSG_EINVAL_INTERNAL: &[u8] = b"EINVAL: internal error";
static MSG_EINVAL_INVALID_DATA: &[u8] = b"EINVAL: invalid data";
static MSG_EINVAL_DECOMPRESSION: &[u8] = b"EINVAL: decompression failed";
static MSG_EINVAL_FILE_PATH: &[u8] = b"EINVAL: file path too long";
static MSG_EINVAL_TEMP_DIR: &[u8] = b"EINVAL: temporary file not in temp dir";
static MSG_EINVAL_TEMP_NAME: &[u8] = b"EINVAL: temporary file not named correctly";
static MSG_EINVAL_UNSUPPORTED_FORMAT: &[u8] = b"EINVAL: unsupported format";
static MSG_EINVAL_UNSUPPORTED_MEDIUM: &[u8] = b"EINVAL: unsupported medium";
static MSG_EINVAL_UNSUPPORTED_DEPTH: &[u8] = b"EINVAL: unsupported pixel depth";
static MSG_EINVAL_DIMENSIONS_REQUIRED: &[u8] = b"EINVAL: dimensions required";
static MSG_EINVAL_DIMENSIONS_TOO_LARGE: &[u8] = b"EINVAL: dimensions too large";

pub(crate) fn encode_error(err: ImageError) -> Response {
    let msg: &[u8] = match err {
        ImageError::OutOfMemory => MSG_ENOMEM,
        ImageError::InternalError => MSG_EINVAL_INTERNAL,
        ImageError::InvalidData => MSG_EINVAL_INVALID_DATA,
        ImageError::DecompressionFailed => MSG_EINVAL_DECOMPRESSION,
        ImageError::FilePathTooLong => MSG_EINVAL_FILE_PATH,
        ImageError::TemporaryFileNotInTempDir => MSG_EINVAL_TEMP_DIR,
        ImageError::TemporaryFileNotNamedCorrectly => MSG_EINVAL_TEMP_NAME,
        ImageError::UnsupportedFormat => MSG_EINVAL_UNSUPPORTED_FORMAT,
        ImageError::UnsupportedMedium => MSG_EINVAL_UNSUPPORTED_MEDIUM,
        ImageError::UnsupportedDepth => MSG_EINVAL_UNSUPPORTED_DEPTH,
        ImageError::DimensionsRequired => MSG_EINVAL_DIMENSIONS_REQUIRED,
        ImageError::DimensionsTooLarge => MSG_EINVAL_DIMENSIONS_TOO_LARGE,
    };
    Response::with_message(msg)
}

pub(crate) struct ExecContext {
    pub storage: *mut ImageStorage,
    pub terminal: *mut c_void,
}

pub(crate) fn execute(
    ctx: *mut ExecContext,
    cmd: *const Command,
) -> Option<Response> {
    if ctx.is_null() || cmd.is_null() {
        return None;
    }

    let ctx_ref = unsafe { &*ctx };
    let cmd_ref = unsafe { &*cmd };
    let storage = unsafe { &mut *ctx_ref.storage };

    if !storage.enabled() {
        return None;
    }

    let mut quiet = cmd_ref.quiet;

    let resp: Option<Response> = match &cmd_ref.control {
        CommandControl::Query(_) => Some(execute_query(ctx, cmd_ref)),
        CommandControl::Display(_) => Some(execute_display(ctx, cmd_ref)),
        CommandControl::Delete(ref d) => Some(execute_delete(ctx, cmd_ref, *d)),

        CommandControl::Transmit(_) | CommandControl::TransmitAndDisplay { .. } => {
            if let Some(loading) = storage.loading_mut() {
                match cmd_ref.quiet {
                    CommandQuiet::No => quiet = loading.quiet,
                    CommandQuiet::Ok => { loading.quiet = CommandQuiet::Ok; },
                    CommandQuiet::Failures => { loading.quiet = CommandQuiet::Failures; },
                }
            }
            Some(execute_transmit(ctx, cmd_ref))
        },

        CommandControl::TransmitAnimationFrame(_) |
        CommandControl::ControlAnimation(_) |
        CommandControl::ComposeAnimation(_) => {
            Some(Response::with_message(MSG_EINVAL_UNIMPLEMENTED))
        },
    };

    match resp {
        Some(r) => {
            match quiet {
                CommandQuiet::No => {
                    if r.empty() { None } else { Some(r) }
                },
                CommandQuiet::Ok => {
                    if r.ok() { None } else { Some(r) }
                },
                CommandQuiet::Failures => None,
            }
        },
        None => None,
    }
}

fn execute_query(
    ctx: *mut ExecContext,
    cmd: &Command,
) -> Response {
    let t = match cmd.transmission() {
        Some(t) => t,
        None => return Response::with_message(MSG_EINVAL_INVALID_DATA),
    };

    if t.image_id == 0 {
        return Response::with_message(MSG_EINVAL_IMAGE_ID_REQUIRED);
    }

    let mut result = Response::new();
    result.id = t.image_id;
    result.image_number = t.image_number;
    result.placement_id = t.placement_id;

    let ctx_ref = unsafe { &*ctx };
    let storage = unsafe { &*ctx_ref.storage };

    let mut loading = match LoadingImage::init_from_command(
        cmd,
        storage.image_limits,
        ptr::null_mut(),
        0,
    ) {
        Ok(l) => l,
        Err(err) => {
            let mut r = encode_error(err);
            r.id = result.id;
            r.image_number = result.image_number;
            r.placement_id = result.placement_id;
            return r;
        },
    };
    let _ = &mut loading;

    result
}

fn execute_transmit(
    ctx: *mut ExecContext,
    cmd: &Command,
) -> Response {
    let t = match cmd.transmission() {
        Some(t) => t,
        None => return Response::with_message(MSG_EINVAL_INVALID_DATA),
    };

    let mut result = Response::new();
    result.id = t.image_id;
    result.image_number = t.image_number;
    result.placement_id = t.placement_id;

    if t.image_id > 0 && t.image_number > 0 {
        return Response::with_message(MSG_EINVAL_ID_AND_NUMBER_EXCLUSIVE);
    }

    let ctx_ref = unsafe { &*ctx };
    let storage = unsafe { &mut *ctx_ref.storage };

    match load_and_add_image(storage, cmd) {
        Ok(load_result) => {
            if load_result.more {
                return Response::new();
            }

            if load_result.implicit_id {
                return Response::new();
            }

            result.id = load_result.image_id;
            result
        },
        Err(err) => {
            let mut r = encode_error(err);
            r.id = result.id;
            r.image_number = result.image_number;
            r.placement_id = result.placement_id;
            r
        },
    }
}

struct LoadResult {
    image_id: u32,
    more: bool,
    implicit_id: bool,
}

fn load_and_add_image(
    storage: &mut ImageStorage,
    cmd: &Command,
) -> Result<LoadResult, ImageError> {
    let t = match cmd.transmission() {
        Some(t) => t,
        None => return Err(ImageError::InvalidData),
    };

    if let Some(loading) = storage.loading_mut() {
        loading.add_data(cmd.data_ptr, cmd.data_len)?;

        if t.more_chunks {
            return Ok(LoadResult {
                image_id: loading.image.id,
                more: true,
                implicit_id: false,
            });
        }

        let mut img = loading.complete()?;

        if img.id == 0 {
            img.id = storage.next_image_id;
            storage.next_image_id = storage.next_image_id.wrapping_add(1);
            if img.number == 0 {
                img.implicit_id = true;
            }
        }

        let image_id = img.id;
        let implicit_id = img.implicit_id;
        storage.add_image(img)?;
        storage.clear_loading();

        return Ok(LoadResult {
            image_id,
            more: false,
            implicit_id,
        });
    }

    let mut loading = LoadingImage::init_from_command(
        cmd,
        storage.image_limits,
        storage.scratch_buf_mut(),
        storage.scratch_cap(),
    )?;

    if loading.image.id == 0 {
        loading.image.id = storage.next_image_id;
        storage.next_image_id = storage.next_image_id.wrapping_add(1);
        if loading.image.number == 0 {
            loading.image.implicit_id = true;
        }
    }

    if t.more_chunks {
        storage.set_loading(loading);
        let id = match storage.loading_mut() {
            Some(l) => l.image.id,
            None => 0,
        };
        return Ok(LoadResult {
            image_id: id,
            more: true,
            implicit_id: false,
        });
    }

    let mut img = loading.complete()?;
    let image_id = img.id;
    let implicit_id = img.implicit_id;
    storage.add_image(img)?;

    Ok(LoadResult {
        image_id,
        more: false,
        implicit_id,
    })
}

fn execute_display(
    ctx: *mut ExecContext,
    cmd: &Command,
) -> Response {
    let d = match cmd.display() {
        Some(d) => d,
        None => return Response::with_message(MSG_EINVAL_INVALID_DATA),
    };

    if d.image_id == 0 && d.image_number == 0 {
        return Response::with_message(MSG_EINVAL_ID_OR_NUMBER_REQUIRED);
    }

    let mut result = Response::new();
    result.id = d.image_id;
    result.image_number = d.image_number;
    result.placement_id = d.placement_id;

    let ctx_ref = unsafe { &*ctx };
    let storage = unsafe { &mut *ctx_ref.storage };

    let img = if d.image_id != 0 {
        storage.image_by_id(d.image_id)
    } else {
        storage.image_by_number(d.image_number)
    };

    let img = match img {
        Some(i) => i,
        None => {
            return Response::with_message(MSG_ENOENT_IMAGE_NOT_FOUND);
        },
    };

    result.id = img.id;

    if d.virtual_placement {
        if d.parent_id > 0 {
            return Response::with_message(MSG_EINVAL_VIRTUAL_PARENT);
        }
    }

    let placement = Placement {
        location_is_virtual: d.virtual_placement,
        pin_node: ptr::null_mut(),
        pin_x: 0,
        pin_y: 0,
        x_offset: d.x_offset,
        y_offset: d.y_offset,
        source_x: d.x,
        source_y: d.y,
        source_width: d.width,
        source_height: d.height,
        columns: d.columns,
        rows: d.rows,
        z: d.z,
    };

    match storage.add_placement(img.id, result.placement_id, placement) {
        Ok(()) => {},
        Err(_) => {
            return encode_error(ImageError::OutOfMemory);
        },
    }

    result
}

fn execute_delete(
    ctx: *mut ExecContext,
    _cmd: &Command,
    delete: Delete,
) -> Response {
    let ctx_ref = unsafe { &*ctx };
    let storage = unsafe { &mut *ctx_ref.storage };

    storage.delete(delete);

    Response::new()
}
