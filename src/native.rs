//! Ventana macOS por FFI directo a APIs del sistema; no usa crates.
#![allow(non_snake_case)]
use std::{
    ffi::{c_char, c_void, CString},
    ptr,
};
type Id = *mut c_void;
type Sel = *mut c_void;
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Size {
    width: f64,
    height: f64,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Rect {
    origin: Point,
    size: Size,
}
#[link(name = "objc")]
extern "C" {
    fn objc_getClass(name: *const c_char) -> Id;
    fn sel_registerName(name: *const c_char) -> Sel;
    fn objc_msgSend();
}
#[link(name = "AppKit", kind = "framework")]
extern "C" {}
#[link(name = "QuartzCore", kind = "framework")]
extern "C" {}
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFDataCreate(allocator: Id, bytes: *const u8, length: isize) -> Id;
    fn CFRelease(value: Id);
}
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGDataProviderCreateWithCFData(data: Id) -> Id;
    fn CGColorSpaceCreateDeviceRGB() -> Id;
    fn CGImageCreate(
        width: usize,
        height: usize,
        bitsPerComponent: usize,
        bitsPerPixel: usize,
        bytesPerRow: usize,
        space: Id,
        bitmapInfo: u32,
        provider: Id,
        decode: *const f64,
        interpolate: bool,
        intent: i32,
    ) -> Id;
    fn CGImageRelease(image: Id);
    fn CGDataProviderRelease(provider: Id);
    fn CGColorSpaceRelease(space: Id);
}
unsafe fn class(s: &str) -> Id {
    objc_getClass(CString::new(s).unwrap().as_ptr())
}
unsafe fn sel(s: &str) -> Sel {
    sel_registerName(CString::new(s).unwrap().as_ptr())
}
unsafe fn msg0<R>(o: Id, s: &str) -> R {
    let f: unsafe extern "C" fn(Id, Sel) -> R =
        std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
    f(o, sel(s))
}
unsafe fn msg1<A, R>(o: Id, s: &str, a: A) -> R {
    let f: unsafe extern "C" fn(Id, Sel, A) -> R =
        std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
    f(o, sel(s), a)
}
unsafe fn string(s: &str) -> Id {
    msg1(
        class("NSString"),
        "stringWithUTF8String:",
        CString::new(s).unwrap().as_ptr(),
    )
}
pub enum Event {
    Down(Point),
    Drag(Point),
    Up(Point),
    Scroll(f64),
    Key(u16),
    KeyUp(u16),
    FocusLost,
    Quit,
}
pub struct Window {
    app: Id,
    window: Id,
    view: Id,
    layer: Id,
    pool: Id,
    width: usize,
    height: usize,
    mode: Id,
}
impl Window {
    pub fn new(width: usize, height: usize) -> Self {
        unsafe {
            let pool = msg0(class("NSAutoreleasePool"), "new");
            let app = msg0(class("NSApplication"), "sharedApplication");
            let _: bool = msg1(app, "setActivationPolicy:", 0isize);
            let allocated: Id = msg0(class("NSWindow"), "alloc");
            let init: unsafe extern "C" fn(Id, Sel, Rect, usize, usize, bool) -> Id =
                std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
            let window = init(
                allocated,
                sel("initWithContentRect:styleMask:backing:defer:"),
                Rect {
                    origin: Point { x: 0., y: 0. },
                    size: Size {
                        width: 1100.,
                        height: 756.,
                    },
                },
                1 | 2 | 4,
                2,
                false,
            );
            let _: () = msg1(window, "setReleasedWhenClosed:", false);
            let _: () = msg1(window, "setTitle:", string("Diorama"));
            let _: () = msg0(window, "center");
            let view = msg0(window, "contentView");
            let _: () = msg1(view, "setWantsLayer:", true);
            let layer = msg0(view, "layer");
            let _: () = msg1(layer, "setMagnificationFilter:", string("linear"));
            let _: () = msg1(layer, "setContentsGravity:", string("resize"));
            let _: () = msg0(app, "finishLaunching");
            let _: () = msg1(window, "makeKeyAndOrderFront:", ptr::null_mut::<c_void>());
            let _: () = msg1(app, "activateIgnoringOtherApps:", true);
            let mode = string("kCFRunLoopDefaultMode");
            let _: Id = msg0(mode, "retain");
            Self {
                app,
                window,
                view,
                layer,
                pool,
                width,
                height,
                mode,
            }
        }
    }
    pub fn present(&self, rgba: &[u8]) {
        assert_eq!(rgba.len(), self.width * self.height * 4);
        unsafe {
            let data = CFDataCreate(ptr::null_mut(), rgba.as_ptr(), rgba.len() as isize);
            let provider = CGDataProviderCreateWithCFData(data);
            let space = CGColorSpaceCreateDeviceRGB();
            let image = CGImageCreate(
                self.width,
                self.height,
                8,
                32,
                self.width * 4,
                space,
                1,
                provider,
                ptr::null(),
                false,
                0,
            );
            let _: () = msg1(self.layer, "setContents:", image);
            let _: () = msg0(class("CATransaction"), "flush");
            CGImageRelease(image);
            CGColorSpaceRelease(space);
            CGDataProviderRelease(provider);
            CFRelease(data);
        }
    }
    fn point(&self, event: Id) -> Point {
        unsafe {
            let p: Point = msg0(event, "locationInWindow"); // Window content size is fixed, so no architecture-dependent NSRect return is needed.
            let _ = self.view;
            Point {
                x: p.x / 1100. * self.width as f64,
                y: (756. - p.y) / 756. * self.height as f64,
            }
        }
    }
    pub fn events(&self) -> Vec<Event> {
        unsafe {
            let pool: Id = msg0(class("NSAutoreleasePool"), "new");
            let mut events = vec![];
            let until: Id = msg0(class("NSDate"), "distantPast");
            let next: unsafe extern "C" fn(Id, Sel, usize, Id, Id, bool) -> Id =
                std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
            loop {
                let event = next(
                    self.app,
                    sel("nextEventMatchingMask:untilDate:inMode:dequeue:"),
                    usize::MAX,
                    until,
                    self.mode,
                    true,
                );
                if event.is_null() {
                    break;
                }
                let kind: usize = msg0(event, "type");
                match kind {
                    1 => events.push(Event::Down(self.point(event))),
                    2 => events.push(Event::Up(self.point(event))),
                    6 => events.push(Event::Drag(self.point(event))),
                    10 => events.push(Event::Key(msg0(event, "keyCode"))),
                    11 => events.push(Event::KeyUp(msg0(event, "keyCode"))),
                    22 => events.push(Event::Scroll(msg0(event, "scrollingDeltaY"))),
                    _ => {}
                }
                let _: () = msg1(self.app, "sendEvent:", event);
            }
            let active: bool = msg0(self.app, "isActive");
            let key_window: bool = msg0(self.window, "isKeyWindow");
            if !active || !key_window {
                events.push(Event::FocusLost);
            }
            let visible: bool = msg0(self.window, "isVisible");
            if !visible {
                events.push(Event::Quit);
            }
            let _: () = msg0(pool, "drain");
            events
        }
    }
}
impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            let _: () = msg0(self.window, "close");
            let _: () = msg0(self.window, "release");
            let _: () = msg0(self.mode, "release");
            let _: () = msg0(self.pool, "drain");
        }
    }
}
