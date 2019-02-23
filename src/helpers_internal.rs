use crate::{
    ffi,
    View
};

use std::{
    os::raw::{
        c_void
    },
};

// All callbacks that accept take a (view: ULView) argument

pub unsafe fn unpack_closure_view_cb<F>(closure: &mut F) -> (*mut c_void, unsafe extern "C" fn(*mut c_void, View))
    where
        F: FnMut(View),
{
    extern "C" fn trampoline<F>(data: *mut c_void, n: View)
        where
            F: FnMut(View),
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

static msg_parsing_failed: &'static str = "!parsing failed!";

pub unsafe extern "C" fn log_forward_cb(
    user_data: *mut ::std::os::raw::c_void,
    caller: View,
    source: ffi::ULMessageSource,           /* u32 */
    level: ffi::ULMessageLevel,             /* u32 */
    message: ffi::ULString,                 /* *mut C_String aka *mut u8 */
    line_number: ::std::os::raw::c_uint,    /* u32 */
    column_number: ::std::os::raw::c_uint,  /* u32 */
    source_id: ffi::ULString,               /* *mut C_String aka *mut u8 */
) {
    let level = match level {
        ffi::ULMessageLevel_kMessageLevel_Log => "log",
        ffi::ULMessageLevel_kMessageLevel_Warning => "warning",
        ffi::ULMessageLevel_kMessageLevel_Error => "error",
        ffi::ULMessageLevel_kMessageLevel_Debug => "debug",
        ffi::ULMessageLevel_kMessageLevel_Info => "info",
        _ => "unknown",
    };

    let source = match source {
        ffi::ULMessageSource_kMessageSource_XML => "xml",
        ffi::ULMessageSource_kMessageSource_JS => "js",
        ffi::ULMessageSource_kMessageSource_Network => "network",
        ffi::ULMessageSource_kMessageSource_ConsoleAPI => "consoleapi",
        ffi::ULMessageSource_kMessageSource_Storage => "storage",
        ffi::ULMessageSource_kMessageSource_AppCache => "appcache",
        ffi::ULMessageSource_kMessageSource_Rendering => "rendering",
        ffi::ULMessageSource_kMessageSource_CSS => "css",
        ffi::ULMessageSource_kMessageSource_Security => "security",
        ffi::ULMessageSource_kMessageSource_ContentBlocker => "contentblocker",
        ffi::ULMessageSource_kMessageSource_Other => "other",
        _ => "unknown",
    };

    let message = match String::from_utf16(std::slice::from_raw_parts_mut(
        ffi::ulStringGetData(message),
        ffi::ulStringGetLength(message),
    )) {
        Ok(msg) => msg,
        Err(_) => msg_parsing_failed.to_string(),
    };

    let source_id = match String::from_utf16(std::slice::from_raw_parts_mut(
        ffi::ulStringGetData(source_id),
        ffi::ulStringGetLength(source_id),
    )) {
        Ok(src) => src,
        Err(_) => msg_parsing_failed.to_string(),
    };

    println!(
        "[{}] [{}] {} ({}:{}:{})",
        level, source, message, source_id, line_number, column_number
    );
}
