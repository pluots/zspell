// Special build script is needed to link to python C source on mac

fn main() {
    pyo3_build_config::add_extension_module_link_args();
}
