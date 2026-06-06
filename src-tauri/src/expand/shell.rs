use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellError {
    Disabled,
    Timeout,
    NonUtf8,
    Spawn(String),
    Exit(Option<i32>),
}

pub trait ShellBackend: Send + Sync {
    fn run(&self, args: &[String], cwd: &Path, timeout: Duration) -> Result<String, ShellError>;
}

#[derive(Default)]
pub struct NoopShellBackend;

impl ShellBackend for NoopShellBackend {
    fn run(&self, _args: &[String], _cwd: &Path, _timeout: Duration) -> Result<String, ShellError> {
        Err(ShellError::Disabled)
    }
}

#[derive(Default)]
pub struct TokioShellBackend;

impl ShellBackend for TokioShellBackend {
    fn run(&self, args: &[String], cwd: &Path, timeout: Duration) -> Result<String, ShellError> {
        let args = args.to_vec();
        let cwd = cwd.to_path_buf();

        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| ShellError::Spawn(error.to_string()))?;
            runtime.block_on(run_command(args, cwd, timeout))
        })
        .join()
        .unwrap_or_else(|_| Err(ShellError::Spawn("shell worker panicked".to_string())))
    }
}

async fn run_command(args: Vec<String>, cwd: PathBuf, timeout: Duration) -> Result<String, ShellError> {
    if args.is_empty() {
        return Err(ShellError::Spawn("missing shell argv".to_string()));
    }

    let mut command = tokio::process::Command::new(&args[0]);
    command
        .args(&args[1..])
        .current_dir(cwd)
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    let child = command
        .spawn()
        .map_err(|error| ShellError::Spawn(error.to_string()))?;

    #[cfg(windows)]
    let _job = assign_job_object(child.id())
        .map_err(|error| ShellError::Spawn(error.to_string()))?;

    let output = match tokio::time::timeout(timeout, child.wait_with_output()).await {
        Ok(Ok(output)) => output,
        Ok(Err(error)) => return Err(ShellError::Spawn(error.to_string())),
        Err(_) => return Err(ShellError::Timeout),
    };

    if !output.status.success() {
        return Err(ShellError::Exit(output.status.code()));
    }

    decode_stdout(output.stdout)
}

pub fn decode_stdout(stdout: Vec<u8>) -> Result<String, ShellError> {
    let text = String::from_utf8(stdout).map_err(|_| ShellError::NonUtf8)?;
    Ok(text.trim_end().to_string())
}

#[cfg(windows)]
struct OwnedHandle(windows::Win32::Foundation::HANDLE);

#[cfg(windows)]
impl Drop for OwnedHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = windows::Win32::Foundation::CloseHandle(self.0);
        }
    }
}

#[cfg(windows)]
fn assign_job_object(pid: Option<u32>) -> Result<OwnedHandle, String> {
    use windows::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject,
        JobObjectExtendedLimitInformation, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_SET_QUOTA, PROCESS_TERMINATE,
    };

    let pid = pid.ok_or_else(|| "spawned process missing pid".to_string())?;
    unsafe {
        let job = CreateJobObjectW(None, None).map_err(|error| error.to_string())?;
        let job = OwnedHandle(job);

        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        SetInformationJobObject(
            job.0,
            JobObjectExtendedLimitInformation,
            &limits as *const _ as *const _,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
        .map_err(|error| error.to_string())?;

        let process =
            OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, false, pid).map_err(|error| error.to_string())?;
        let process = OwnedHandle(process);
        AssignProcessToJobObject(job.0, process.0).map_err(|error| error.to_string())?;

        Ok(job)
    }
}
