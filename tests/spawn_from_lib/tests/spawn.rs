use std::{
    fs::File,
    io::{Read, Seek},
    os::fd::OwnedFd,
};

use spawn_worker::{spawn::{SpawnCfg, Target}, WaitStatus};
use test_spawn_from_lib::secret_main_loop;

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
        .target(Target::Trampoline(secret_main_loop))
        .stdin(File::open("/dev/null").unwrap())
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
        .unwrap();

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    match child.wait().unwrap() {
        WaitStatus::Exited(_, s) => assert_eq!(0, s),
        _ => unreachable!("shouldn't happen"),
    }

    assert_eq!(
        "stderr_works_as_expected",
        rewind_and_read_fd(stderr).unwrap()
    );
    assert_eq!(
        "stdout_works_as_expected",
        rewind_and_read_fd(stdout).unwrap()
    );
}
