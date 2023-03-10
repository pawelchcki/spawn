use std::{
    ffi::CString,
    fs::File,
    io::{Read, Seek},
    os::fd::OwnedFd,
};

use nix::sys::wait::WaitStatus;
use spawn_worker::spawn::*;

fn rewind_and_read_fd(fd: OwnedFd) -> anyhow::Result<String> {
    let mut file = File::try_from(fd)?;
    file.rewind()?;
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();

    Ok(buf)
}

#[test]
fn test_spawning_trampoline_worker() {
    let stdout = tempfile::tempfile().unwrap();
    let stderr = tempfile::tempfile().unwrap();

    let mut child = SpawnCfg::new()
        .target(Target::ManualTrampoline(
            CString::new("__dummy_mirror_test").unwrap(),
            CString::new("symbol_name").unwrap(),
        ))
        .stdin(File::open("/dev/null").unwrap())
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
        .unwrap();

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    //wait for process exit
    let status = child.wait().unwrap();

    match status {
        WaitStatus::Exited(_, s) => assert_eq!(0, s),

        others => {
            eprintln!("{}", rewind_and_read_fd(stderr).unwrap());
            panic!("unexpected exit status = {:?}", others)
        }
    }

    assert_eq!(
        "__dummy_mirror_test symbol_name",
        rewind_and_read_fd(stdout).unwrap()
    );
}
