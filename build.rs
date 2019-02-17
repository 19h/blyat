fn main() {
//    println!("cargo:rustc-flags=-L framework=/usr/local/Frameworks/");
//    println!("cargo:rustc-flags=-L dylib=/usr/local/dylib/");
//    println!("cargo:rustc-link-search=framework=/usr/local/Frameworks/");
//    println!("cargo:rustc-link-search=dylib=/usr/local/lib/");
    println!("cargo:rustc-link-lib=framework=Ultralight");
    println!("cargo:rustc-link-lib=framework=WebCore");
}
