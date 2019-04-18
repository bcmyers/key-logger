use std::env;
use std::path::Path;

fn main() {
    pkg_config::probe_library("x11").unwrap();
    pkg_config::probe_library("xkbcommon").unwrap();

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .layout_tests(true)
        .whitelist_function("XGetInputFocus")
        .whitelist_function("XLookupString")
        .whitelist_function("XNextEvent")
        .whitelist_function("XOpenDisplay")
        .whitelist_function("XSelectInput")
        .whitelist_function("XSetErrorHandler")
        .whitelist_function("xkb_keysym_get_name")
        .whitelist_type("KeySym")
        .whitelist_type("Window")
        .whitelist_type("XComposeStatus")
        .whitelist_type("XEvent")
        .whitelist_var("FocusChangeMask")
        .whitelist_var("FocusOut")
        .whitelist_var("KeyPress")
        .whitelist_var("KeyPressMask")
        .generate()
        .expect("Unable to generate bindings");

    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("bindings.rs");
    bindings
        .write_to_file(&path)
        .expect("Unable to write bindings");

    let path = Path::new(".").join("bindings.rs");
    bindings
        .write_to_file(&path)
        .expect("Unable to write bindings");
}
