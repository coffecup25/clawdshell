use std::process::{Command, ExitStatus, Stdio};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

/// Run a command via fallback shell using -c. Supports POSIX positional args.
pub fn run_command_via_shell(
    shell: &str,
    command: &str,
    extra_args: &[String],
) -> Result<ExitStatus, std::io::Error> {
    Command::new(shell)
        .arg("-c")
        .arg(command)
        .args(extra_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
}

pub fn spawn_tool(tool_path: &str, args: &[String]) -> Result<ExitStatus, std::io::Error> {
    Command::new(tool_path)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
}

pub fn spawn_fallback_shell(shell: &str) -> Result<ExitStatus, std::io::Error> {
    #[cfg(unix)]
    {
        let err = Command::new(shell).exec();
        Err(err)
    }
    #[cfg(windows)]
    {
        Command::new(shell)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
    }
}

pub fn forward_to_fallback(shell: &str, args: &[String]) -> Result<ExitStatus, std::io::Error> {
    Command::new(shell)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
}

#[cfg(unix)]
pub mod signals {
    use std::sync::atomic::{AtomicI32, Ordering};

    pub static CHILD_PID: AtomicI32 = AtomicI32::new(0);

    pub fn setup_signal_forwarding() {
        unsafe {
            libc::signal(libc::SIGHUP, sighup_handler as libc::sighandler_t);
            libc::signal(libc::SIGWINCH, sigwinch_handler as libc::sighandler_t);
        }
    }

    pub fn set_child_pid(pid: u32) {
        CHILD_PID.store(pid as i32, Ordering::Relaxed);
    }

    extern "C" fn sighup_handler(_sig: libc::c_int) {
        let pid = CHILD_PID.load(Ordering::Relaxed);
        if pid > 0 {
            unsafe { libc::kill(pid, libc::SIGHUP); }
        }
        std::process::exit(1);
    }

    extern "C" fn sigwinch_handler(_sig: libc::c_int) {
        let pid = CHILD_PID.load(Ordering::Relaxed);
        if pid > 0 {
            unsafe { libc::kill(pid, libc::SIGWINCH); }
        }
    }
}

#[cfg(unix)]
pub fn setup_signal_forwarding() { signals::setup_signal_forwarding(); }

#[cfg(windows)]
pub fn setup_signal_forwarding() {}

#[cfg(unix)]
pub fn set_child_pid(pid: u32) { signals::set_child_pid(pid); }

#[cfg(windows)]
pub fn set_child_pid(_pid: u32) {}
