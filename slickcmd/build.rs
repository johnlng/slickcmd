fn main() {
    println!("cargo:rustc-link-lib=dylib:+verbatim=slickcmd/res/slickcmd.res");
}
