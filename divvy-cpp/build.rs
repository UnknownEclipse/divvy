fn main() {
    cc::Build::new()
        .cpp(true)
        .flag_if_supported("-std=c++17")
        .file("src/ffi.cpp")
        .compile("divvy-cpp-ffi");
}
