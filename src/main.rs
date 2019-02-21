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

fn main() {
    let mut ul = Ultralight::new(None);

    ul.view(1920, 1080, false);

    ul.setFinishLoadingCallback(|_view| println!("loaded!"));
    ul.setDOMReadyCallback(|_view| println!("loaded!"));

    ul.updateUntilLoaded();

    ul.render();

    ul.writePNGToFile("output.png");

    println!("hello");
}
