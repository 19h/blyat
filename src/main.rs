#![feature(try_trait)]
#[allow(
    non_camel_case_types,
    non_upper_case_globals,
    non_snake_case
)]

mod ffi;
pub mod helpers;

use helpers::{
    evaluateScript,
    setJSObjectProperty,
    createJSFunction,
};

#[macro_use]
use std::{
    borrow::{
        Borrow,
        BorrowMut,
    },
    cell::{
        RefCell
    },
    option::NoneError,
    os::raw::{
        c_int,
        c_void
    },
    sync::{
        Arc,
        Mutex
    },
};

mod helpers_internal;
use helpers_internal::{
    unpack_closure_view_cb,
    unpack_closure_hook_cb
};

pub type Renderer = ffi::ULRenderer;
pub type View = ffi::ULView;

struct Ultralight {
    renderer: Renderer,
    view: Option<View>,
}

impl Ultralight {
    fn new(renderer: Option<Renderer>) -> Ultralight {
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

    fn view(&mut self, width: u32, height: u32, transparent: bool) {
        unsafe {
            self.view = Some(ffi::ulCreateView(self.renderer, width, height, transparent));
        }
    }

    fn update(&mut self) {
        unsafe {
            ffi::ulUpdate(self.renderer);
        }
    }

    fn updateUntilLoaded(&mut self) -> Result<(), NoneError> {
        unsafe {
            while ffi::ulViewIsLoading(self.view?) {
                ffi::ulUpdate(self.renderer);
            }
        }

        Ok(())
    }

    fn render(&mut self) {
        unsafe {
            ffi::ulRender(self.renderer);
        }
    }

    fn setFinishLoadingCallback<T>(&mut self, mut cb: T) -> Result<(), NoneError>
        where T: FnMut(ffi::ULView)
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

    fn setDOMReadyCallback<T>(&mut self, mut cb: T) -> Result<(), NoneError>
        where T: FnMut(ffi::ULView)
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

    fn createFunction<T>(
        &mut self,
        name: &'static str,
        mut hook: &mut T
    ) -> Result<ffi::JSObjectRef, NoneError>
        where T: helpers::HookFnMut {
        Ok(
            createJSFunction(
                self.view?,
                name,
                hook
            )
        )
    }

    fn setJSObjectProperty(
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

    fn evaluateScript(
        &mut self,
        name: &'static str,
    ) -> Result<ffi::JSValueRef, NoneError> {
        Ok(evaluateScript(self.view?, name))
    }

    fn writePNGToFile(
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

    fn isLoading(&self) -> bool {
        match self.view {
            Some(view) => unsafe {
                ffi::ulViewIsLoading(view)
            },
            None => false
        }
    }
}

//struct Foo {
//
//}
//
//impl Foo {
//    fn new() -> Foo {
//        Foo {}
//    }
//
//    unsafe fn run (self) {
//        let config = ffi::ulCreateConfig();
//        let renderer = ffi::ulCreateRenderer(config);
//
//        let view = ffi::ulCreateView(renderer, 1920, 1080, false);
//
//        let mut styla_ready = Arc::new(Mutex::new(false));
//
//        {
//            let mut loaded_cb = move |view: ffi::ULView| {
//                println!("loaded");
//            };
//
//            let mut dom_loaded_cb = |view: ffi::ULView| {
//                println!("dom ready");
//            };
//
//            let (cb_ld_closure, cb_ld_callback) = unpack_closure_view_cb(&mut loaded_cb);
//            let (cb_dom_ld_closure, cb_dom_ld_callback) = unpack_closure_view_cb(&mut dom_loaded_cb);
//
//            ffi::ulViewSetFinishLoadingCallback(view, Some(cb_ld_callback), cb_ld_closure);
//            ffi::ulViewSetDOMReadyCallback(view, Some(cb_dom_ld_callback), cb_dom_ld_closure);
//        }
//
//        {
//            let strdy_mtx = styla_ready.clone();
//
//            let mut hook = move |
//                ctx: ffi::JSContextRef,
//                function: ffi::JSObjectRef,
//                thisObject: ffi::JSObjectRef,
//                argumentCount: usize,
//                arguments: *const ffi::JSValueRef,
//                exception: *mut ffi::JSValueRef,
//            | {
//                println!("hook called!");
//
//                *strdy_mtx.lock().unwrap() = true;
//                ffi::JSValueMakeNumber(ctx, 1f64)
//            };
//
//            let (hook_cl, hook_cb) = unpack_closure_hook_cb(&mut hook);
//
//            let classname_str = std::ffi::CString::new("SomeClass").unwrap();
//
//            let mut jsclassdef = ffi::JSClassDefinition {
//                version: 0,
//                attributes: 0,
//                className: classname_str.as_ptr(),
//                parentClass: 0 as ffi::JSClassRef,
//                staticValues: 0 as *const ffi::JSStaticValue,
//                staticFunctions: 0 as *const ffi::JSStaticFunction,
//                initialize: None,
//                hasProperty: None,
//                getProperty: None,
//                setProperty: None,
//                deleteProperty: None,
//                getPropertyNames: None,
//                callAsConstructor: None,
//                hasInstance: None,
//                convertToType: None,
//                finalize: None,
//                callAsFunction: Some(hook_cb),
//                // need to implement drop!
//                //finalize: Some(|| std::mem::drop(jsclass)),
//            };
//
//            let jsclass = ffi::JSClassCreate(
//                &mut jsclassdef
//            );
//
//            let jsgctx = ffi::ulViewGetJSContext(view);
//            let def_obj_ref = ffi::JSContextGetGlobalObject(jsgctx);
//
//            let nafu = ffi::JSObjectMake(jsgctx, jsclass, hook_cl);
//
//            ffi::JSObjectSetProperty(
//                jsgctx,
//                def_obj_ref,
//                ffi::JSStringCreateWithUTF8CString(
//                    std::ffi::CString::new(
//                        "global_spotfire_hook"
//                    ).unwrap().as_ptr()
//                ),
//                nafu,
//                0,
//                0 as *mut *const ffi::OpaqueJSValue
//            );
//
//            let jsvr = ffi::JSEvaluateScript(
//                jsgctx,
//                ffi::JSStringCreateWithUTF8CString(
//                    std::ffi::CString::new(
//                        "window.styla={callbacks:[{render:global_spotfire_hook}]};"
//                    ).unwrap().as_ptr()
//                ),
//                def_obj_ref,
//                0 as *mut ffi::OpaqueJSString,
//                ffi::kJSPropertyAttributeNone as i32,
//                0 as *mut *const ffi::OpaqueJSValue
//            );
//        }
//
//        {
//            let url_str = std::ffi::CString::new(
//                "https://magazine-display.prod.us.magalog.net/prophet/area/nae-en/home"
//                // "https://magazine-display.canary.eu.magalog.net/prophet/area/nae-en/home"
//            ).unwrap();
//
//            let url = ffi::ulCreateString(
//                url_str.as_ptr()
//            );
//
//            ffi::ulViewLoadURL(view, url);
//        }
//
//        //while !has_loaded {
//        while ffi::ulViewIsLoading(view) {
//            ffi::ulUpdate(renderer);
//        }
//
//        ffi::ulRender(renderer);
//
//        println!("yolo");
//
//        ffi::ulBitmapWritePNG(
//            ffi::ulViewGetBitmap( view ),
//            std::ffi::CString::new("output.png").unwrap().as_ptr()
//        );
//    }
//}

fn main() {
//    unsafe {
//        Foo::new().run();
//    }

    let mut ul = Ultralight::new(None);

    ul.view(1920, 1080, false);

    ul.setFinishLoadingCallback(|_view| println!("loaded!"));
    ul.setDOMReadyCallback(|_view| println!("loaded!"));

    ul.updateUntilLoaded();

    ul.render();

    ul.writePNGToFile("output.png");

    println!("hello");
}
