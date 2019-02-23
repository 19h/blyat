use crate::ffi;
use crate::helpers_internal::unpack_closure_hook_cb;

pub fn create_js_function<T> (
    view: crate::View,
    name: &'static str,
    mut hook: &mut T
) -> ffi::JSObjectRef
    where T: FnMut(
        ffi::JSContextRef,
        ffi::JSObjectRef,
        ffi::JSObjectRef,
        usize,
        *const ffi::JSValueRef,
        *mut ffi::JSValueRef,
    ) -> ffi::JSValueRef
{
    unsafe {
        let (
            hook_closure,
            hook_function
        ) = unpack_closure_hook_cb(&mut hook);

        let classname_str = std::ffi::CString::new(name).unwrap();

        let jsclassdef = ffi::JSClassDefinition {
            version: 0,
            attributes: 0,
            className: classname_str.as_ptr(),
            parentClass: std::ptr::null_mut() as ffi::JSClassRef,
            staticValues: std::ptr::null() as *const ffi::JSStaticValue,
            staticFunctions: std::ptr::null() as *const ffi::JSStaticFunction,
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
            &jsclassdef
        );

        let (jsgctx, ..) = getJSContextFromView(view);

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

pub fn set_js_object_property(
    view: crate::View,
    name: &'static str,
    object: ffi::JSObjectRef
) {
    unsafe {
        let (jsgctx, jsgctx_object) = getJSContextFromView(view);

        let c_name = std::ffi::CString::new(
            name
        ).unwrap();

        let propertyName = ffi::JSStringCreateWithUTF8CString(
            c_name.as_ptr()
        );

        ffi::JSObjectSetProperty(
            jsgctx,
            jsgctx_object,
            propertyName,
            object,
            0,
            std::ptr::null_mut() as *mut *const ffi::OpaqueJSValue
        );
    }
}

// "window.styla={callbacks:[{render:global_spotfire_hook}]};"

pub fn evaluate_script(
    view: crate::View,
    script: &'static str
) -> ffi::JSValueRef {
    unsafe {
        let (jsgctx, jsgctx_object) = getJSContextFromView(view);

        let script_c_str = std::ffi::CString::new(
            script
        ).unwrap();

        ffi::JSEvaluateScript(
            jsgctx,
            ffi::JSStringCreateWithUTF8CString(
                script_c_str.as_ptr()
            ),
            jsgctx_object,
            std::ptr::null_mut() as *mut ffi::OpaqueJSString,
            ffi::kJSPropertyAttributeNone as i32,
            std::ptr::null_mut() as *mut *const ffi::OpaqueJSValue
        )
    }
}
