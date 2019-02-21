#[allow(
    non_camel_case_types,
    non_upper_case_globals,
    non_snake_case
)]

use crate::ffi;
use crate::helpers_internal::unpack_closure_hook_cb;

pub trait HookFnMut: FnMut(
    ffi::JSContextRef,
    ffi::JSObjectRef,
    ffi::JSObjectRef,
    usize,
    *const ffi::JSValueRef,
    *mut ffi::JSValueRef,
) -> ffi::JSValueRef {}

pub fn createJSFunction<T> (
    view: crate::View,
    name: &'static str,
    mut hook: &mut T
) -> ffi::JSObjectRef
    where T: HookFnMut
{
    unsafe {
        let (
            hook_closure,
            hook_function
        ) = unpack_closure_hook_cb(&mut hook);

        let classname_str = std::ffi::CString::new(name).unwrap();

        let mut jsclassdef = ffi::JSClassDefinition {
            version: 0,
            attributes: 0,
            className: classname_str.as_ptr(),
            parentClass: 0 as ffi::JSClassRef,
            staticValues: 0 as *const ffi::JSStaticValue,
            staticFunctions: 0 as *const ffi::JSStaticFunction,
            initialize: None,
            hasProperty: None,
            getProperty: None,
            setProperty: None,
            deleteProperty: None,
            getPropertyNames: None,
            callAsConstructor: None,
            hasInstance: None,
            convertToType: None,
            finalize: None,
            callAsFunction: Some(hook_function),
            // need to implement drop!
            //finalize: Some(|| std::mem::drop(jsclass)),
        };

        let jsclass = ffi::JSClassCreate(
            &mut jsclassdef
        );

        let (jsgctx, ..) = getJSContextFromView(view);
//
        ffi::JSObjectMake(
            jsgctx,
            jsclass,
            hook_closure
        )
    }
}

pub fn getJSContextFromView(
    view: crate::View
) -> (ffi::JSContextRef, ffi::JSObjectRef) {
    unsafe {
        let jsgctx = ffi::ulViewGetJSContext(view);
        let jsgctx_object = ffi::JSContextGetGlobalObject(jsgctx);

        (jsgctx, jsgctx_object)
    }
}

pub fn setJSObjectProperty(
    view: crate::View,
    name: &'static str,
    object: ffi::JSObjectRef
) {
    unsafe {
        let (jsgctx, jsgctx_object) = getJSContextFromView(view);

        let propertyName = ffi::JSStringCreateWithUTF8CString(
            std::ffi::CString::new(
                name
            ).unwrap().as_ptr()
        );

        ffi::JSObjectSetProperty(
            jsgctx,
            jsgctx_object,
            propertyName,
            object,
            0,
            0 as *mut *const ffi::OpaqueJSValue
        );
    }
}

// "window.styla={callbacks:[{render:global_spotfire_hook}]};"

pub fn evaluateScript(
    view: crate::View,
    script: &'static str
) -> ffi::JSValueRef {
    unsafe {
        let (jsgctx, jsgctx_object) = getJSContextFromView(view);

        ffi::JSEvaluateScript(
            jsgctx,
            ffi::JSStringCreateWithUTF8CString(
                std::ffi::CString::new(
                    script
                ).unwrap().as_ptr()
            ),
            jsgctx_object,
            0 as *mut ffi::OpaqueJSString,
            ffi::kJSPropertyAttributeNone as i32,
            0 as *mut *const ffi::OpaqueJSValue
        )
    }
}
