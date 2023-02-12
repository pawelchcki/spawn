fn main() {
    cc_utils::BinaryBuild::new()
        .file("src/trampoline.c")
        .warnings(true)
        .warnings_into_errors(true)
        .try_compile("trampoline.bin")
        .unwrap();
}
