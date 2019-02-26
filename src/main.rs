#![feature(try_trait,unboxed_closures)]
#![allow(
    non_camel_case_types,
    non_upper_case_globals,
    non_snake_case,
    dead_code,
    unused_variables,
    unused_must_use
)]

mod ffi;
pub mod helpers;
pub mod config;

use helpers::{
    evaluate_script,
    set_js_object_property,
    create_js_function,
};

use std::{
    cell::{
        RefCell
    },
    option::NoneError,
    os::raw::c_void,
    time::Duration,
};

mod helpers_internal;
use helpers_internal::{
    log_forward_cb,
    unpack_closure_view_cb,
};

use crate::ffi::JSValueRef;

pub type Renderer = ffi::ULRenderer;
pub type View = ffi::ULView;
pub type Config = config::UltralightConfig;

pub struct Ultralight {
    config: Config,
    renderer: Renderer,
    view: Option<View>,
}

impl Ultralight {
    pub fn new(config: Option<Config>, renderer: Option<Renderer>) -> Ultralight {
        let ulconfig = match config {
            Some(config) => config,
            None => Config::new()
        };

        let used_renderer = match renderer {
            Some(renderer) => renderer,
            None => {
                unsafe {
                    ffi::ulCreateRenderer(ulconfig.to_ulconfig())
                }
            }
        };

        Ultralight {
            config: ulconfig,
            renderer: used_renderer,
            view: None
        }
    }

    pub fn view(&mut self, width: u32, height: u32, transparent: bool) {
        unsafe {
            self.view = Some(ffi::ulCreateView(self.renderer, width, height, transparent));
        }
    }

    pub fn load_url(&mut self, url: &'static str) -> Result<(), NoneError> {
        unsafe {
            let url_str = std::ffi::CString::new(
                url
            ).unwrap();

            let url = ffi::ulCreateString(
                url_str.as_ptr()
            );

            ffi::ulViewLoadURL(self.view?, url);
        }

        Ok(())
    }

    pub fn load_html(&mut self, code: &'static str) -> Result<(), NoneError> {
        unsafe {
            let code_str = std::ffi::CString::new(
                code
            ).unwrap();

            let code = ffi::ulCreateString(
                code_str.as_ptr()
            );

            ffi::ulViewLoadHTML(self.view?, code);
        }

        Ok(())
    }

    pub fn update(&mut self) {
        unsafe {
            ffi::ulUpdate(self.renderer);
        }
    }

    pub fn update_until_loaded(&mut self) -> Result<(), NoneError> {
        unsafe {
            while ffi::ulViewIsLoading(self.view?) {
                ffi::ulUpdate(self.renderer);
            }
        }

        Ok(())
    }

    pub fn render(&mut self) {
        unsafe {
            ffi::ulRender(self.renderer);
        }
    }

    pub fn scroll(&mut self, delta_x: i32, delta_y: i32) -> Result<(), NoneError> {
        unsafe {
            let scrollEvent = ffi::ulCreateScrollEvent(
                ffi::ULScrollEventType_kScrollEventType_ScrollByPixel,
                delta_x,
                delta_y
            );

            ffi::ulViewFireScrollEvent(self.view?, scrollEvent);

            ffi::ulDestroyScrollEvent(scrollEvent);

            Ok(())
        }
    }

    pub fn get_scroll_height(&mut self) -> Result<f64, NoneError> {
        unsafe {
            let (jsgctx, _) = helpers::getJSContextFromView(self.view?);

            Ok(ffi::JSValueToNumber(
                jsgctx,
                self.evaluate_script("document.body.scrollHeight").unwrap(),
                std::ptr::null_mut()
            ))
        }
    }

    pub fn set_finish_loading_callback<T>(&mut self, mut cb: T) -> Result<(), NoneError>
        where T: FnMut(View)
    {
        let view = self.view?;

        unsafe {
            let (
                cb_closure,
                cb_function
            ) = unpack_closure_view_cb(&mut cb);

            ffi::ulViewSetFinishLoadingCallback(
                view,
                Some(cb_function),
                cb_closure
            );
        }

        Ok(())
    }

    pub fn set_dom_ready_callback<T>(&mut self, mut cb: T) -> Result<(), NoneError>
        where T: FnMut(View)
    {
        let view = self.view?;

        unsafe {
            let (
                cb_closure,
                cb_function
            ) = unpack_closure_view_cb(&mut cb);

            ffi::ulViewSetDOMReadyCallback(
                view,
                Some(cb_function),
                cb_closure
            );
        }

        Ok(())
    }

    pub fn create_function<T>(
        &mut self,
        name: &'static str,
        hook: &mut T
    ) -> Result<ffi::JSObjectRef, NoneError>
        where T: FnMut(
            ffi::JSContextRef,
            ffi::JSObjectRef,
            ffi::JSObjectRef,
            usize,
            *const ffi::JSValueRef,
            *mut ffi::JSValueRef,
        ) -> ffi::JSValueRef
    {
        Ok(
            create_js_function(
                self.view?,
                name,
                hook
            )
        )
    }

    pub fn set_js_object_property(
        &mut self,
        name: &'static str,
        object: ffi::JSObjectRef
    ) -> Result<(), NoneError> {
        set_js_object_property(
            self.view?,
            name,
            object
        );

        Ok(())
    }

    pub fn evaluate_script(
        &mut self,
        script: &'static str,
    ) -> Result<ffi::JSValueRef, NoneError> {
        Ok(evaluate_script(self.view?, script))
    }

    pub fn get_raw_pixels(&mut self) -> Result<Vec<u8>, NoneError> {
        unsafe {
            let bitmap_obj = ffi::ulViewGetBitmap( self.view? );

            let bitmap = ffi::ulBitmapLockPixels(bitmap_obj);
            let bitmap_size = ffi::ulBitmapGetSize(bitmap_obj);

            let bitmap_raw = std::slice::from_raw_parts_mut(
                bitmap as *mut u8,
                bitmap_size,
            );

            ffi::ulBitmapUnlockPixels(bitmap_obj);

            Ok(bitmap_raw.to_vec())
        }
    }

    pub fn write_png_to_file(
        &mut self,
        file_name: &'static str,
    ) -> Result<bool, NoneError> {
        unsafe {
            let bitmap_obj = ffi::ulViewGetBitmap( self.view? );

            let bitmap = ffi::ulBitmapLockPixels(bitmap_obj);
            let bitmap_size = ffi::ulBitmapGetSize(bitmap_obj);

            let bitmap_raw = std::slice::from_raw_parts_mut(
                bitmap as *mut u8,
                bitmap_size,
            );

            let fn_c_str = std::ffi::CString::new(file_name).unwrap();

            Ok(
                ffi::ulBitmapWritePNG(
                    bitmap_obj,
                    fn_c_str.as_ptr()
                )
            )
        }
    }

    pub fn is_loading(&self) -> bool {
        match self.view {
            Some(view) => unsafe {
                ffi::ulViewIsLoading(view)
            },
            None => false
        }
    }

    pub fn log_to_stdout(&mut self) -> Result<(), NoneError> {
        unsafe {
            ffi::ulViewSetAddConsoleMessageCallback(
                self.view?,
                Some(log_forward_cb),
                std::ptr::null_mut() as *mut c_void
            );
        }

        Ok(())
    }
}

thread_local! {
    static STYLA_LOADED: RefCell<bool> = RefCell::new(false);
}

fn main() {
    let config = Config::new();

    let mut ul = Ultralight::new(Some(config), None);

    ul.view(1920, 1080, false);
    ul.log_to_stdout();

    ul.load_url("https://www.foundationdb.org/blog/announcing-record-layer/");
    //ul.load_url("https://magazines.styla.com/prophet/area/nae-en/all");
    //ul.load_url("https://legalizepsychedelics.com");
    //ul.load_url("https://r3.dtr.is");
    //ul.load_url("https://www.styla.com/landing-pages/");
    //ul.load_url("https://psychonautwiki.org/wiki/LSD");
    //ul.load_url("https://en.wikipedia.org/wiki/Love");

    ul.set_finish_loading_callback(|_view| println!("loaded!"));
    ul.set_dom_ready_callback(|_view| println!("dom ready!"));

    ul.update_until_loaded();

    {
        let mut hook = |
            ctx: ffi::JSContextRef,
            function: ffi::JSObjectRef,
            thisObject: ffi::JSObjectRef,
            argumentCount: usize,
            arguments: *const ffi::JSValueRef,
            exception: *mut ffi::JSValueRef,
        | -> JSValueRef {
            println!("hook was called!");

            STYLA_LOADED.with(|f| *f.borrow_mut() = true);

            unsafe {
                ffi::JSValueMakeNumber(ctx, 0f64)
            }
        };

        if let Ok(func) = ul.create_function("hook", &mut hook) {
            ul.set_js_object_property("hook", func);
        }

        ul.evaluate_script(r#"
            console.log(123);
            //window.setTimeout(hook, 3000);
            hook();
        "#);
    }

    let mut styla_loaded = false;

    while !styla_loaded {
        ul.update();

        STYLA_LOADED.with(|f| styla_loaded = *f.borrow());

        std::thread::sleep(Duration::from_millis(10));
    }

    unsafe {
        let mut frames: Vec<u8> = Vec::new();

        ul.render();

        let width = 1920u32;
        let height = 1080u32;

        let bpp = 4u32;
        let row_bytes = width * bpp;

        let scroll_height = ul.get_scroll_height().unwrap();
        let frame_modulo = scroll_height % height as f64;

        let snapshot_num = (scroll_height / height as f64) as usize;

        let extra_frame = match frame_modulo {
            0.0 => 0,
            _ => 1
        };

        let last_frame_skip_rows = (height - frame_modulo as u32) as usize;

        let size = row_bytes * scroll_height as u32;

        for i in 0..(snapshot_num + extra_frame) {
            if let Ok(mut pixels) = ul.get_raw_pixels() {
                let mut pixelbuf = {
                    if i == snapshot_num && frame_modulo != 0.0 {
                        pixels.iter()
                            .skip(last_frame_skip_rows * (bpp * width) as usize)
                            .map(|vec| *vec)
                            .collect::<Vec<u8>>()
                    } else {
                        pixels
                    }
                };

                frames.append(
                    &mut pixelbuf
                );
            }

            ul.scroll(0, -1i32 * height as i32);
            ul.render();
        }

        let xbitmap = ffi::ulCreateBitmapFromPixels(
            width,
            scroll_height as u32,
            ffi::ULBitmapFormat_kBitmapFormat_RGBA8,
            row_bytes as u32,
            frames.as_ptr() as *const c_void,
            size as usize,
            false,
        );

        let fn_c_str = std::ffi::CString::new("test.png").unwrap();

        ffi::ulBitmapWritePNG(
            xbitmap,
            fn_c_str.as_ptr()
        );
    }

    ul.write_png_to_file("output.png");

    println!("finish");
}
