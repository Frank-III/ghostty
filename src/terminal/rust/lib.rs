#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(dead_code, unused_imports)]

use core::ffi::{c_int, c_void};
use core::mem;
use core::panic::PanicInfo;
use core::ptr;


mod early;
mod constants;
mod terminal;
mod render;
mod input;
mod selection;
mod kitty_graphics;
mod kitty_image;
mod kitty_placement;
mod mouse_encode;
mod simple;
mod size_report;
mod style;
mod event;
mod paste;
mod focus;
mod modes;
mod osc;
mod key_encode;
mod sys;
mod grid_ref;
mod cell;
mod row;
mod sgr;
mod color;
mod allocator;
mod build_info;

pub(crate) use early::*;
pub(crate) use constants::*;
pub(crate) use terminal::*;
pub(crate) use render::*;
pub(crate) use input::*;
pub(crate) use selection::*;
pub(crate) use kitty_graphics::*;
pub(crate) use kitty_image::*;
pub(crate) use kitty_placement::*;
pub(crate) use mouse_encode::*;
pub(crate) use simple::*;
pub(crate) use size_report::*;
pub(crate) use style::*;
pub(crate) use event::*;
pub(crate) use paste::*;
pub(crate) use focus::*;
pub(crate) use modes::*;
pub(crate) use osc::*;
pub(crate) use key_encode::*;
pub(crate) use sys::*;
pub(crate) use grid_ref::*;
pub(crate) use cell::*;
pub(crate) use row::*;
pub(crate) use sgr::*;
pub(crate) use color::*;
pub(crate) use allocator::*;
pub(crate) use build_info::*;

#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    loop {}
}
