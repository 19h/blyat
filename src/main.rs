#[allow(non_camel_case_types,non_upper_case_globals,non_snake_case)]

#[macro_use]
use std::{
    borrow::{
        Borrow,
        BorrowMut,
    },
    thread_local,
    sync::{
        Arc,
        Mutex
    },
};

pub mod ffi;

thread_local! {
    static UL_CTX_LOAD_CALLBACK: Mutex<Option<Box<
        FnMut(
            ffi::ULView
        ) + Send
    >>> = Default::default();

    static UL_CTX_DOM_READY_CALLBACK: Mutex<Option<Box<
        FnMut(
            ffi::ULView
        ) + Send
    >>> = Default::default();

    static JS_HOOK_CALLBACK: Mutex<Option<Box<
        FnMut(
            ffi::JSContextRef,
            ffi::JSObjectRef,
            ffi::JSObjectRef,
            usize,
            *const ffi::JSValueRef,
            *mut ffi::JSValueRef,
        ) + Send
    >>> = Default::default();
}

unsafe extern "C" fn hook_callback (
    ctx: ffi::JSContextRef,
    _function: ffi::JSObjectRef,
    _thisObject: ffi::JSObjectRef,
    _argumentCount: usize,
    _arguments: *const ffi::JSValueRef,
    _exception: *mut ffi::JSValueRef,
) -> ffi::JSValueRef {
    println!("hook_callback callback called");

    return ffi::JSValueMakeNumber(ctx, 1123 as f64);
}

unsafe extern "C" fn load_callback_trampoline (view: ffi::ULView) {
    UL_CTX_LOAD_CALLBACK.with(|f|
        match *(*f.borrow()).lock().unwrap() {
            Some(ref mut callback) => callback(view),
            None => panic!("Calling callback failed")
        }
    );

    println!("{:?}", std::thread::current().id());
    println!("loaded");
}

unsafe extern "C" fn dom_ready_callback_trampoline (view: ffi::ULView) {
    UL_CTX_DOM_READY_CALLBACK.with(|f|
        match *(*f.borrow()).lock().unwrap() {
            Some(ref mut callback) => callback(view),
            None => panic!("Calling callback failed")
        }
    );
}

struct Foo {

}

impl Foo {
    fn new() -> Foo {
        Foo {}
    }

    unsafe fn run (self) {
        let config = ffi::ulCreateConfig();
        let renderer = ffi::ulCreateRenderer(config);

        let view = ffi::ulCreateView(renderer, 1280, 768, false);

        let url_str = std::ffi::CString::new(
            "https://magazine-display.prod.us.magalog.net/prophet/area/nae-en/home"
            // "https://magazine-display.canary.eu.magalog.net/prophet/area/nae-en/home"
            // "https://google.de"
        ).unwrap();

        let url = ffi::ulCreateString(
            url_str.as_ptr()
        );

        let mut has_loaded = Arc::new(Mutex::new(false));
        let has_loaded_mtx = has_loaded.clone();

        let cbl = move |view: ffi::ULView| *has_loaded_mtx.lock().unwrap() = true;

        let dom_loaded_cb = |view: ffi::ULView| {
            println!("dom ready");

            let jsc = ffi::ulViewGetJSContext(view);
            let val = ffi::ulViewEvaluateScript(
                view,
                ffi::ulCreateString(
                    std::ffi::CString::new(
                        "document.body.innerHTML"
                    ).unwrap().as_ptr()
                )
            );

            if ffi::JSValueGetType(jsc, val) == ffi::JSType_kJSTypeString {
                let jsgctx = ffi::JSContextGetGlobalContext(jsc);

                let def_obj_ref = ffi::JSContextGetGlobalObject(jsgctx);

                let fxcb = ffi::JSObjectMakeFunctionWithCallback(
                    jsgctx,
                    0 as *mut ffi::OpaqueJSString,
                    Some(hook_callback)
                );

                ffi::JSObjectSetProperty(
                    jsgctx,
                    def_obj_ref,
                    ffi::JSStringCreateWithUTF8CString(
                        std::ffi::CString::new(
                            "global_spotfire_hook"
                        ).unwrap().as_ptr()
                    ),
                    fxcb,
                    0,
                    0 as *mut *const ffi::OpaqueJSValue
                );

                let jsvr = ffi::JSEvaluateScript(
                    jsgctx,
                    ffi::JSStringCreateWithUTF8CString(
                        std::ffi::CString::new(
                            "window.styla={callbacks:[{render:global_spotfire_hook}]};"
                        ).unwrap().as_ptr()
                    ),
                    def_obj_ref,
                    0 as *mut ffi::OpaqueJSString,
                    ffi::kJSPropertyAttributeNone as i32,
                    0 as *mut *const ffi::OpaqueJSValue
                );
            }
        };

        UL_CTX_LOAD_CALLBACK.with(|mut f| {
            *(*f.borrow_mut()).lock().unwrap() = Some(Box::new(cbl))
        });

        UL_CTX_DOM_READY_CALLBACK.with(|mut f| {
            *(*f.borrow_mut()).lock().unwrap() = Some(Box::new(dom_loaded_cb))
        });

        ffi::ulViewSetFinishLoadingCallback(view, Some(load_callback_trampoline));
        ffi::ulViewSetDOMReadyCallback(view, Some(dom_ready_callback_trampoline));

        ffi::ulViewLoadURL(view, url);

        while !*has_loaded.lock().unwrap() {
            ffi::ulUpdate(renderer);
        }

        ffi::ulRender(renderer);

        ffi::ulBitmapWritePNG(
            ffi::ulCreateBitmap(1280, 768, ffi::ULBitmapFormat_kBitmapFormat_RGBA8),
            std::ffi::CString::new("output.png").unwrap().as_ptr()
        );
    }
}

fn main() {
    unsafe {
        Foo::new().run();
    }

    println!("hello");
}