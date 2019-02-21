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

use helpers::{
    evaluateScript,
    setJSObjectProperty,
    createJSFunction,
};

use std::{
    cell::{
        RefCell
    },
    option::NoneError,
};

mod helpers_internal;
use helpers_internal::{
    unpack_closure_view_cb,
};

use crate::ffi::JSValueRef;

pub type Renderer = ffi::ULRenderer;
pub type View = ffi::ULView;

pub struct Ultralight {
    renderer: Renderer,
    view: Option<View>,
}

impl Ultralight {
    pub fn new(renderer: Option<Renderer>) -> Ultralight {
        let used_renderer = match renderer {
            Some(renderer) => renderer,
            None => {
                unsafe {
                    let config = ffi::ulCreateConfig();

                    ffi::ulCreateRenderer(config)
                }
            }
        };

        Ultralight {
            renderer: used_renderer,
            view: None
        }
    }

    pub fn view(&mut self, width: u32, height: u32, transparent: bool) {
        unsafe {
            self.view = Some(ffi::ulCreateView(self.renderer, width, height, transparent));
        }
    }

    pub fn loadUrl(&mut self, url: &'static str) -> Result<(), NoneError> {
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

    pub fn loadHTML(&mut self, code: &'static str) -> Result<(), NoneError> {
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

    pub fn updateUntilLoaded(&mut self) -> Result<(), NoneError> {
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

    pub fn setFinishLoadingCallback<T>(&mut self, mut cb: T) -> Result<(), NoneError>
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

    pub fn setDOMReadyCallback<T>(&mut self, mut cb: T) -> Result<(), NoneError>
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

    pub fn createFunction<T>(
        &mut self,
        name: &'static str,
        mut hook: &mut T
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
            createJSFunction(
                self.view?,
                name,
                hook
            )
        )
    }

    pub fn setJSObjectProperty(
        &mut self,
        name: &'static str,
        object: ffi::JSObjectRef
    ) -> Result<(), NoneError> {
        setJSObjectProperty(
            self.view?,
            name,
            object
        );

        Ok(())
    }

    pub fn evaluateScript(
        &mut self,
        script: &'static str,
    ) -> Result<ffi::JSValueRef, NoneError> {
        Ok(evaluateScript(self.view?, script))
    }

    pub fn writePNGToFile(
        &mut self,
        file_name: &'static str,
    ) -> Result<bool, NoneError> {
        unsafe {
            Ok(
                ffi::ulBitmapWritePNG(
                    ffi::ulViewGetBitmap( self.view? ),
                    std::ffi::CString::new(file_name).unwrap().as_ptr()
                )
            )
        }
    }

    pub fn isLoading(&self) -> bool {
        match self.view {
            Some(view) => unsafe {
                ffi::ulViewIsLoading(view)
            },
            None => false
        }
    }
}

thread_local! {
    static STYLA_LOADED: RefCell<bool> = RefCell::new(false);
}

fn main() {
    let mut ul = Ultralight::new(None);

    ul.view(1920, 1080, false);

    //ul.loadUrl("https://magazine-display.prod.us.magalog.net/prophet/area/nae-en/home");
    ul.loadUrl("https://www.foundationdb.org/blog/announcing-record-layer/");

    ul.setFinishLoadingCallback(|_view| println!("loaded!"));
    ul.setDOMReadyCallback(|_view| println!("dom ready!"));

    ul.updateUntilLoaded();

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

        match ul.createFunction("hook", &mut hook) {
            Ok(func) => {
                ul.setJSObjectProperty("hook", func);
            },
            _ => {}
        }

        //ul.evaluateScript("window.styla.callbacks=[{render:hook}]");
        ul.evaluateScript("window.setTimeout(hook, 3000)");
    }

    let mut styla_loaded = false;

    while !styla_loaded {
        ul.update();

        STYLA_LOADED.with(|f| styla_loaded = *f.borrow());

        std::thread::sleep_ms(10);
    }

    ul.render();

    ul.writePNGToFile("output.png");

    println!("finish");
}
