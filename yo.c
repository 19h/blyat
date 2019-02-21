#include "Ultralight/CAPI.h"

bool has_loaded = false;

void load_callback(void* user_data, ULView view) {
    printf("load");
    has_loaded = true;
}

JSValueRef fncb (JSContextRef ctx, JSObjectRef function, JSObjectRef thisObject, size_t argumentCount, const JSValueRef arguments[], JSValueRef* exception) {
    printf("fncb callback called");

    return JSValueMakeNumber(ctx, (double) 1123);
}

void dom_ready(void* user_data, ULView view) {
    printf("dom ready....");

    JSContextRef jsc = ulViewGetJSContext(view);
    JSValueRef val = ulViewEvaluateScript(
        view,
        ulCreateString("document.body.innerHTML")
    );

    if ( JSValueGetType(jsc, val) == kJSTypeString ) {
        JSStringRef jssr = JSValueToStringCopy(jsc, val, NULL);
        size_t jslen = JSStringGetLength(jssr);

        printf("%zu", jslen);

        char* jsstr = malloc(jslen);
        JSStringGetUTF8CString(jssr, jsstr, jslen);

        JSGlobalContextRef jsgctx = JSContextGetGlobalContext(jsc);

        JSObjectRef def_obj_ref = JSContextGetGlobalObject(jsgctx);

        JSObjectRef fxcb = JSObjectMakeFunctionWithCallback(
            jsgctx,
            NULL,
            &fncb
        );

        JSObjectSetProperty(
            jsgctx,
            def_obj_ref,
            JSStringCreateWithUTF8CString("xxxx"),
            fxcb,
            0,
            NULL
        );

        free(jsstr);

        JSValueRef jsvr = JSEvaluateScript(
            jsgctx,
            JSStringCreateWithUTF8CString(
                "window.xxx={callbacks:[{render:xxxx}]};"
            ),
            def_obj_ref,
            NULL,
            kJSPropertyAttributeNone,
            NULL
        );
    }
}

int main() {
    ULConfig config = ulCreateConfig();
    ULRenderer renderer = ulCreateRenderer(config);

    ULView view = ulCreateView(renderer, 1280, 768, false);

    ulViewSetFinishLoadingCallback(view, load_callback, 0);
    ulViewSetDOMReadyCallback(view, dom_ready, 0);

    ULString url = ulCreateString("https://google.com");

    ulViewLoadURL(view, url);

    while ( !has_loaded ) {
        ulUpdate(renderer);
    }

    ulRender(renderer);

    ulBitmapWritePNG(
        ulCreateBitmap(1280, 768, kBitmapFormat_RGBA8),
        "output.png"
    );

    return 0;
}
