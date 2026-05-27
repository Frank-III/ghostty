use core::ffi::{c_char, c_int, c_uint, c_ulong, c_void};
use core::ptr;

use crate::kitty_graphics_command::*;

// ---------------------------------------------------------------------------
// zlib decompression via system libz (extern "C" - no crate dependency)
// ---------------------------------------------------------------------------

#[repr(C)]
struct ZStream {
    next_in: *const u8,
    avail_in: c_uint,
    total_in: c_ulong,
    next_out: *mut u8,
    avail_out: c_uint,
    total_out: c_ulong,
    msg: *const c_char,
    state: *mut c_void,
    zalloc: *mut c_void,
    zfree: *mut c_void,
    opaque: *mut c_void,
    data_type: c_int,
    adler: c_ulong,
    reserved: c_ulong,
}

const Z_OK: c_int = 0;
const Z_STREAM_END: c_int = 1;
const Z_FINISH: c_int = 4;
const ZLIB_VERSION: &[u8] = b"1.2.13\0";

extern "C" {
    fn inflateInit2_(
        strm: *mut ZStream,
        windowBits: c_int,
        version: *const u8,
        stream_size: c_int,
    ) -> c_int;
    fn inflate(strm: *mut ZStream, flush: c_int) -> c_int;
    fn inflateEnd(strm: *mut ZStream) -> c_int;
}

#[cfg(not(target_os = "windows"))]
mod zlib_mmap {
    use super::*;

    const PROT_READ: c_int = 0x1;
    const PROT_WRITE: c_int = 0x2;
    const MAP_PRIVATE: c_int = 0x02;
    const MAP_ANONYMOUS: c_int = if cfg!(target_os = "macos") { 0x1000 } else { 0x20 };
    const MAP_FAILED: *mut c_void = !0 as *mut c_void;

    extern "C" {
        fn mmap(
            addr: *mut c_void,
            len: usize,
            prot: c_int,
            flags: c_int,
            fd: c_int,
            offset: i64,
        ) -> *mut c_void;
        fn munmap(addr: *mut c_void, len: usize) -> c_int;
    }

    pub unsafe fn temp_alloc(n: usize) -> *mut u8 {
        let p = unsafe {
            mmap(ptr::null_mut(), n, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0)
        };
        if p == MAP_FAILED || p.is_null() {
            return ptr::null_mut();
        }
        p as *mut u8
    }

    pub unsafe fn temp_free(p: *mut u8, n: usize) {
        if !p.is_null() {
            unsafe { munmap(p as *mut c_void, n); }
        }
    }
}

#[cfg(target_os = "windows")]
mod zlib_mmap {
    use super::*;

    const MEM_COMMIT: u32 = 0x00001000;
    const MEM_RESERVE: u32 = 0x00002000;
    const MEM_RELEASE: u32 = 0x00008000;
    const PAGE_READWRITE: u32 = 0x04;

    extern "system" {
        fn VirtualAlloc(addr: *mut c_void, size: usize, atype: u32, protect: u32) -> *mut c_void;
        fn VirtualFree(addr: *mut c_void, size: usize, ftype: u32) -> i32;
    }

    pub unsafe fn temp_alloc(n: usize) -> *mut u8 {
        let p = unsafe { VirtualAlloc(ptr::null_mut(), n, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE) };
        if p.is_null() { ptr::null_mut() } else { p as *mut u8 }
    }

    pub unsafe fn temp_free(p: *mut u8, _n: usize) {
        if !p.is_null() {
            unsafe { VirtualFree(p as *mut c_void, 0, MEM_RELEASE); }
        }
    }
}

pub(crate) const MAX_DIMENSION: u32 = 10000;
pub(crate) const MAX_SIZE: usize = 400 * 1024 * 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum ImageError {
    InternalError = 0,
    InvalidData = 1,
    DecompressionFailed = 2,
    DimensionsRequired = 3,
    DimensionsTooLarge = 4,
    FilePathTooLong = 5,
    TemporaryFileNotInTempDir = 6,
    TemporaryFileNotNamedCorrectly = 7,
    UnsupportedFormat = 8,
    UnsupportedMedium = 9,
    UnsupportedDepth = 10,
    OutOfMemory = 11,
}

#[derive(Clone, Copy)]
pub(crate) struct LoadingLimits {
    pub file: bool,
    pub temporary_file: bool,
    pub shared_memory: bool,
}

impl LoadingLimits {
    pub(crate) fn all() -> Self {
        Self {
            file: true,
            temporary_file: true,
            shared_memory: true,
        }
    }

    pub(crate) fn direct_only() -> Self {
        Self {
            file: false,
            temporary_file: false,
            shared_memory: false,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Image {
    pub id: u32,
    pub number: u32,
    pub width: u32,
    pub height: u32,
    pub format: TransmissionFormat,
    pub compression: TransmissionCompression,
    pub data_ptr: *const u8,
    pub data_len: usize,
    pub transmit_time_ns: u64,
    pub implicit_id: bool,
}

impl Image {
    pub(crate) fn new() -> Self {
        Self {
            id: 0,
            number: 0,
            width: 0,
            height: 0,
            format: TransmissionFormat::Rgb,
            compression: TransmissionCompression::None,
            data_ptr: ptr::null(),
            data_len: 0,
            transmit_time_ns: 0,
            implicit_id: false,
        }
    }

    pub(crate) fn without_data(&self) -> Image {
        let mut copy = *self;
        copy.data_ptr = ptr::null();
        copy.data_len = 0;
        copy
    }
}

pub(crate) struct LoadingImage {
    pub image: Image,
    pub data_buf: *mut u8,
    pub data_len: usize,
    pub data_cap: usize,
    pub display: Option<Display>,
    pub quiet: CommandQuiet,
}

impl LoadingImage {
    pub(crate) fn new(
        data_buf: *mut u8,
        data_cap: usize,
    ) -> Self {
        Self {
            image: Image::new(),
            data_buf,
            data_len: 0,
            data_cap,
            display: None,
            quiet: CommandQuiet::No,
        }
    }

    pub(crate) fn init_from_command(
        cmd: &Command,
        limits: LoadingLimits,
        data_buf: *mut u8,
        data_cap: usize,
    ) -> Result<Self, ImageError> {
        let t = match cmd.transmission() {
            Some(t) => t,
            None => return Err(ImageError::InvalidData),
        };

        let mut result = Self::new(data_buf, data_cap);
        result.image.id = t.image_id;
        result.image.number = t.image_number;
        result.image.width = t.width;
        result.image.height = t.height;
        result.image.compression = t.compression;
        result.image.format = t.format;
        result.display = cmd.display();
        result.quiet = cmd.quiet;

        if t.medium == TransmissionMedium::Direct {
            result.add_data(cmd.data_ptr, cmd.data_len)?;
            return Ok(result);
        }

        if t.format == TransmissionFormat::Png {
            return Err(ImageError::UnsupportedMedium);
        }

        match t.medium {
            TransmissionMedium::Direct => {},
            TransmissionMedium::File => {
                if !limits.file {
                    return Err(ImageError::UnsupportedMedium);
                }
            },
            TransmissionMedium::TemporaryFile => {
                if !limits.temporary_file {
                    return Err(ImageError::UnsupportedMedium);
                }
            },
            TransmissionMedium::SharedMemory => {
                if !limits.shared_memory {
                    return Err(ImageError::UnsupportedMedium);
                }
            },
        }

        Err(ImageError::UnsupportedMedium)
    }

    pub(crate) fn add_data(&mut self, src: *const u8, src_len: usize) -> Result<(), ImageError> {
        if src_len == 0 {
            return Ok(());
        }

        let new_len = self.data_len.wrapping_add(src_len);
        if new_len > MAX_SIZE {
            return Err(ImageError::InvalidData);
        }
        if new_len > self.data_cap {
            return Err(ImageError::OutOfMemory);
        }

        unsafe {
            ptr::copy_nonoverlapping(
                src,
                self.data_buf.add(self.data_len),
                src_len,
            );
        }
        self.data_len = new_len;
        Ok(())
    }

    pub(crate) fn complete(&mut self) -> Result<Image, ImageError> {
        self.decompress()?;

        if self.image.format == TransmissionFormat::Png {
            self.decode_png()?;
        }

        if self.image.width == 0 || self.image.height == 0 {
            return Err(ImageError::DimensionsRequired);
        }
        if self.image.width > MAX_DIMENSION || self.image.height > MAX_DIMENSION {
            return Err(ImageError::DimensionsTooLarge);
        }

        let bpp = format_bpp(self.image.format) as usize;
        if bpp == 0 {
            return Err(ImageError::InvalidData);
        }
        let expected_len = (self.image.width as usize)
            .wrapping_mul(self.image.height as usize)
            .wrapping_mul(bpp);
        if self.data_len != expected_len {
            return Err(ImageError::InvalidData);
        }

        let mut result = self.image;
        result.data_ptr = self.data_buf;
        result.data_len = self.data_len;
        self.image = Image::new();
        Ok(result)
    }

    fn decompress(&mut self) -> Result<(), ImageError> {
        match self.image.compression {
            TransmissionCompression::None => Ok(()),
            TransmissionCompression::ZlibDeflate => self.decompress_zlib(),
        }
    }

    fn decompress_zlib(&mut self) -> Result<(), ImageError> {
        if self.data_len == 0 || self.data_cap == 0 {
            return Err(ImageError::DecompressionFailed);
        }

        let temp_size = self.data_cap;
        let temp: *mut u8 = unsafe { zlib_mmap::temp_alloc(temp_size) };
        if temp.is_null() {
            return Err(ImageError::OutOfMemory);
        }

        let mut strm: ZStream = unsafe { core::mem::zeroed() };
        strm.next_in = self.data_buf as *const u8;
        strm.avail_in = self.data_len as c_uint;
        strm.next_out = temp;
        strm.avail_out = temp_size as c_uint;

        let result = unsafe {
            let init_ret = inflateInit2_(
                &mut strm,
                15,
                ZLIB_VERSION.as_ptr(),
                core::mem::size_of::<ZStream>() as c_int,
            );
            if init_ret != Z_OK {
                zlib_mmap::temp_free(temp, temp_size);
                return Err(ImageError::DecompressionFailed);
            }

            let inflate_ret = inflate(&mut strm, Z_FINISH);
            inflateEnd(&mut strm);

            if inflate_ret != Z_STREAM_END {
                zlib_mmap::temp_free(temp, temp_size);
                return Err(ImageError::DecompressionFailed);
            }

            let out_len = strm.total_out as usize;
            if out_len > temp_size {
                zlib_mmap::temp_free(temp, temp_size);
                return Err(ImageError::InvalidData);
            }

            ptr::copy_nonoverlapping(temp, self.data_buf, out_len);
            self.data_len = out_len;
            self.image.compression = TransmissionCompression::None;
            Ok(())
        };

        unsafe { zlib_mmap::temp_free(temp, temp_size); }
        result
    }

    fn decode_png(&mut self) -> Result<(), ImageError> {
        Err(ImageError::UnsupportedFormat)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Rect {
    pub top_left_x: u16,
    pub top_left_y: u16,
    pub top_left_node: *mut core::ffi::c_void,
    pub bottom_right_x: u16,
    pub bottom_right_y: u16,
    pub bottom_right_node: *mut core::ffi::c_void,
}

impl Rect {
    pub(crate) fn new() -> Self {
        Self {
            top_left_x: 0,
            top_left_y: 0,
            top_left_node: ptr::null_mut(),
            bottom_right_x: 0,
            bottom_right_y: 0,
            bottom_right_node: ptr::null_mut(),
        }
    }
}
