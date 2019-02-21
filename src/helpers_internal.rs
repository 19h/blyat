use crate::ffi;

use std::{
    os::raw::{
        c_int,
        c_void
    },
};

// All callbacks that accept take a (view: ULView) argument

pub unsafe fn unpack_closure_view_cb<F>(closure: &mut F) -> (*mut c_void, unsafe extern "C" fn(*mut c_void, ffi::ULView))
    where
        F: FnMut(ffi::ULView),
{
    extern "C" fn trampoline<F>(data: *mut c_void, n: ffi::ULView)
        where
            F: FnMut(ffi::ULView),
    {
        let closure: &mut F = unsafe { &mut *(data as *mut F) };
        (*closure)(n);
    }

    (closure as *mut F as *mut c_void, trampoline::<F>)
}

// JSContextHooks
type ClosureHookCallbackSig = unsafe extern "C" fn(
    ffi::JSContextRef,
    ffi::JSObjectRef,
    ffi::JSObjectRef,
    usize,
    *const ffi::JSValueRef,
    *mut ffi::JSValueRef
) -> ffi::JSValueRef;

pub unsafe fn unpack_closure_hook_cb<F>(closure: &mut F) -> (*mut c_void, ClosureHookCallbackSig)
    where
        F: FnMut(
            ffi::JSContextRef,
            ffi::JSObjectRef,
            ffi::JSObjectRef,
            usize,
            *const ffi::JSValueRef,
            *mut ffi::JSValueRef,
        ) -> ffi::JSValueRef,
{
    unsafe extern "C" fn trampoline<F>(
        ctx: ffi::JSContextRef,
        function: ffi::JSObjectRef,
        thisObject: ffi::JSObjectRef,
        argumentCount: usize,
        arguments: *const ffi::JSValueRef,
        exception: *mut ffi::JSValueRef,
    ) -> ffi::JSValueRef
        where
            F: FnMut(
                ffi::JSContextRef,
                ffi::JSObjectRef,
                ffi::JSObjectRef,
                usize,
                *const ffi::JSValueRef,
                *mut ffi::JSValueRef,
            ) -> ffi::JSValueRef,
    {
        let closure: &mut F = &mut *(ffi::JSObjectGetPrivate(function) as *mut F);

        (*closure)(
            ctx,
            function,
            thisObject,
            argumentCount,
            arguments,
            exception
        )
    }

    (closure as *mut F as *mut c_void, trampoline::<F>)
}
