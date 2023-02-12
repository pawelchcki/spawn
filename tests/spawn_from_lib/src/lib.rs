use std::io::Write;

#[no_mangle]
pub extern "C" fn secret_main_loop() {
    print!("stdout_works_as_expected");
    eprint!("stderr_works_as_expected");
    std::io::stdout().flush().unwrap();
    std::io::stderr().flush().unwrap();
}
