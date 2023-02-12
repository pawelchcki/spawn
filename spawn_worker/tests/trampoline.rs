use std::{
    ffi::CString,
    fs::File,
    io::{Read, Seek},
};

use nix::sys::wait::WaitStatus;
use spawn_worker::spawn::*;

#[test]
fn test_spawning_trampoline_worker() {
    let file = tempfile::tempfile().unwrap();

    let mut trampoline = SpawnCfg::new()
        .target(Target::ManualTrampoline(
            CString::new("__dummy_mirror_test").unwrap(),
            CString::new("symbol_name").unwrap(),
        ))
        .stdin(File::open("/dev/null").unwrap())
        .stdout(file)
        .stderr(File::open("/dev/null").unwrap())
        .spawn()
        .unwrap();

    let mut file = File::try_from(trampoline.stdout.take().unwrap()).unwrap();

    match trampoline.wait().unwrap() {
        WaitStatus::Exited(_, s) => assert_eq!(0, s),
        _ => unreachable!("shouldn't happen"),
    }

    file.rewind().unwrap();

    let mut out = String::new();
    file.read_to_string(&mut out).unwrap();
    assert_eq!("__dummy_mirror_test symbol_name", out);
}
