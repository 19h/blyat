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

    pub fn write_png_to_file(
        &mut self,
        file_name: &'static str,
    ) -> Result<bool, NoneError> {
        unsafe {
            let bitmap = ffi::ulViewGetBitmap( self.view? );

            let fn_c_str = std::ffi::CString::new(file_name).unwrap();

            Ok(
                ffi::ulBitmapWritePNG(
                    bitmap,
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

    ul.view(1920, 15000, false);
    ul.log_to_stdout();

    //ul.load_url("https://www.foundationdb.org/blog/announcing-record-layer/");
    //ul.load_url("https://magazines.styla.com/prophet/area/nae-en/all");
    //ul.load_url("https://legalizepsychedelics.com");
    //ul.load_url("https://r3.dtr.is");
    ul.load_url("https://psychonautwiki.org/wiki/LSD");
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
            window.setTimeout(hook, 3000);
        "#);
    }

    let mut styla_loaded = false;

    while !styla_loaded {
        ul.update();

        STYLA_LOADED.with(|f| styla_loaded = *f.borrow());

        std::thread::sleep(Duration::from_millis(10));
    }

    ul.render();

    ul.write_png_to_file("output.png");

    println!("finish");
}
