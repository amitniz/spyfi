extern crate bindgen;
extern crate cc;

fn main() {
    println!("cargo:rerun-if-changed=c_wlan/wlan_com.c");
    cc::Build::new().file("c_wlan/wlan_com.c").compile("c_wlan");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .generate()
        .expect("failed to generate bindings");
    let output_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(output_path.join("bindings.rs"))
        .expect("failed to write bindings");
}
