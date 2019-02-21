fn main() {
    println!("cargo:rustc-flags=-L framework=/usr/local/Frameworks/");
    println!("cargo:rustc-flags=-L dylib=/usr/local/lib/");
    println!("cargo:rustc-link-search=framework=/usr/local/Frameworks/");
    println!("cargo:rustc-link-search=dylib=/usr/local/lib/");
    println!("cargo:rustc-link-lib=framework=Ultralight");
    println!("cargo:rustc-link-lib=framework=WebCore");
}

// bindgen --use-core --impl-debug --impl-partialeq --generate-inline-functions --dump-preprocessed-input --conservative-inline-namespaces --whitelist-function "^UL.*|JS.*|ul.*|WK.*" --whitelist-var "^UL.*|JS.*|ul.*|WK.*" --whitelist-type "^UL.*|JS.*|ul.*|WK.*"
