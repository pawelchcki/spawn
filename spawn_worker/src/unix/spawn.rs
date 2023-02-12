#[cfg(target_os = "linux")]
mod linux {
    use std::{
        ffi::{self, CStr, CString},
        io::{Seek, Write},
        os::fd::AsRawFd,
        ptr::{self},
    };

    use crate::{spawn::Target, Fork, TRAMPOLINE_BIN};

    use super::SpawnCfg;

    fn write_trampoline() -> anyhow::Result<memfd::Memfd> {
        let opts = memfd::MemfdOptions::default();
        let mfd = opts.create("spawn_worker_trampoline")?;

        mfd.as_file().set_len(TRAMPOLINE_BIN.len() as u64)?;
        mfd.as_file().write_all(TRAMPOLINE_BIN)?;
        mfd.as_file().rewind()?;

        Ok(mfd)
    }

    fn spawn_fd<T: AsRawFd>(fd: T, cfg: &SpawnCfg) -> anyhow::Result<Option<libc::pid_t>> {
        let fd = fd.as_raw_fd();
        static PROG_NAME: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"trampoline\0") };
        let mut argv: Vec<*const ffi::c_char> = vec![PROG_NAME.as_ptr()];
        let mut string_store: Vec<CString> = vec![];

        match &cfg.target {
            Target::Trampoline(f) => {
                let (library_path, symbol_name) =
                    match unsafe { crate::get_dl_path_raw(*f as *const libc::c_void) } {
                        (Some(p), Some(n)) => (p, n),
                        _ => return Err(anyhow::format_err!("can't read symbol pointer data")),
                    };

                argv.push(library_path.as_ptr());
                string_store.push(library_path); // ensure the data is kept around until process forks

                argv.push(symbol_name.as_ptr());
                string_store.push(symbol_name);
            }
            Target::ManualTrampoline(library_path, symbol_name) => {
                argv.push(library_path.as_ptr());
                argv.push(symbol_name.as_ptr());
            }
            Target::Fork(_) => todo!(),
            Target::Noop => return Ok(None),
        };

        // list must be null terminated
        argv.push(ptr::null());

        let mut envp: Vec<*const ffi::c_char> = vec![];

        let mut environs: Vec<CString> = vec![];
        if cfg.inherit_env {
            for (k, v) in std::env::vars() {
                environs.push(CString::new(format!("{k}={v}"))?);
            }
        }

        for env in &environs {
            envp.push(env.as_ptr());
        }

        envp.push(ptr::null());

        match unsafe { crate::fork()? } {
            Fork::Parent(child_pid) => Ok(Some(child_pid)),
            Fork::Child => unsafe {
                if let Some(fd) = &cfg.stdin {
                    libc::dup2(fd.as_raw_fd(), libc::STDIN_FILENO);
                }

                if let Some(fd) = &cfg.stdout {
                    libc::dup2(fd.as_raw_fd(), libc::STDOUT_FILENO);
                }

                if let Some(fd) = &cfg.stderr {
                    libc::dup2(fd.as_raw_fd(), libc::STDERR_FILENO);
                }

                // not using nix crate here, as it would allocate args after fork, which will lead to crashes on systems
                // where allocator is not fork+thread safe
                libc::fexecve(fd, argv.as_ptr(), envp.as_ptr());
                std::process::exit(1);
            },
        }
    }

    pub fn spawn_trampoline(cfg: &SpawnCfg) -> anyhow::Result<Option<libc::pid_t>> {
        let fd = write_trampoline()?;
        spawn_fd(fd, cfg)
    }
}

pub enum Target {
    Trampoline(extern "C" fn()),
    ManualTrampoline(CString, CString),
    Fork(fn()),
    Noop,
}

pub struct SpawnCfg {
    stdin: Option<OwnedFd>,
    stderr: Option<OwnedFd>,
    stdout: Option<OwnedFd>,
    target: Target,
    inherit_env: bool,
}

impl SpawnCfg {
    pub fn new() -> Self {
        Self {
            stdin: None,
            stdout: None,
            stderr: None,
            target: Target::Noop,
            inherit_env: true,
        }
    }

    pub fn target(&mut self, target: Target) -> &mut Self {
        self.target = target;
        self
    }

    pub fn stdin<T: Into<OwnedFd>>(&mut self, fd: T) -> &mut Self {
        self.stdin = Some(fd.into());
        self
    }

    pub fn stdout<T: Into<OwnedFd>>(&mut self, fd: T) -> &mut Self {
        self.stdout = Some(fd.into());
        self
    }

    pub fn stderr<T: Into<OwnedFd>>(&mut self, fd: T) -> &mut Self {
        self.stderr = Some(fd.into());
        self
    }

    pub fn spawn(&mut self) -> anyhow::Result<Child> {
        let pid = spawn_trampoline(self)?;

        Ok(Child {
            pid,
            stdin: self.stdin.take(),
            stderr: self.stderr.take(),
            stdout: self.stdout.take(),
        })
    }
}
impl Default for SpawnCfg {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Child {
    pid: Option<libc::pid_t>,
    pub stdin: Option<OwnedFd>,
    pub stderr: Option<OwnedFd>,
    pub stdout: Option<OwnedFd>,
}

impl Child {
    pub fn wait(self) -> anyhow::Result<WaitStatus> {
        let pid = match self.pid {
            Some(pid) => Pid::from_raw(pid),
            None => return Ok(WaitStatus::Exited(Pid::from_raw(0), 0)),
        };

        Ok(nix::sys::wait::waitpid(Some(pid), None)?)
    }
}

use std::{ffi::CString, os::fd::OwnedFd};

#[cfg(target_os = "linux")]
pub use linux::spawn_trampoline;
use nix::{sys::wait::WaitStatus, unistd::Pid};
