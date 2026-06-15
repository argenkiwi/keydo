//! IPC protocol between `keyd` daemon and client subcommands (bind, macro, reload, listen).

use std::io::{self, Read, Write};

use crate::error::KeydoError;

/// Path of the daemon's IPC endpoint: a Unix socket, or a named pipe on Windows.
#[cfg(unix)]
pub const SOCKET_PATH: &str = "/var/run/keyd.socket";
#[cfg(windows)]
pub const SOCKET_PATH: &str = r"\\.\pipe\keydo";

/// Platform stream for one IPC connection. On Windows the named-pipe handle is
/// wrapped in `File`, which provides the same blocking Read/Write semantics.
#[cfg(unix)]
pub type IpcStream = std::os::unix::net::UnixStream;
#[cfg(windows)]
pub type IpcStream = std::fs::File;

/// Maximum payload size for an IPC message, matching C's `struct ipc_message`.
pub const IPC_DATA_SIZE: usize = 4096;

/// IPC message type — numeric values match the C enum exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum IpcMessageType {
    Success     = 0,
    Fail        = 1,
    Bind        = 2,
    Input       = 3,
    Macro       = 4,
    Reload      = 5,
    LayerListen = 6,
}

impl TryFrom<u32> for IpcMessageType {
    type Error = u32;
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::Success),
            1 => Ok(Self::Fail),
            2 => Ok(Self::Bind),
            3 => Ok(Self::Input),
            4 => Ok(Self::Macro),
            5 => Ok(Self::Reload),
            6 => Ok(Self::LayerListen),
            x => Err(x),
        }
    }
}

/// Binary-compatible with C's `struct ipc_message` (64-bit layout):
///   u32 type | u32 timeout | u8\[4096\] data | u64 sz
#[repr(C)]
pub struct IpcMessage {
    pub msg_type: u32,
    pub timeout:  u32,
    pub data:     [u8; IPC_DATA_SIZE],
    pub sz:       u64,
}

// Compile-time guard: if the struct acquires padding the wire format breaks.
const _: () = assert!(
    std::mem::size_of::<IpcMessage>() == 4 + 4 + IPC_DATA_SIZE + 8,
    "IpcMessage layout mismatch: struct has unexpected padding",
);

impl IpcMessage {
    pub fn new(msg_type: IpcMessageType, timeout: u32) -> Self {
        Self {
            msg_type: msg_type as u32,
            timeout,
            data: [0u8; IPC_DATA_SIZE],
            sz: 0,
        }
    }

    pub fn set_data(&mut self, src: &[u8]) {
        let sz = src.len().min(self.data.len());
        self.data[..sz].copy_from_slice(&src[..sz]);
        self.sz = sz as u64;
    }

    /// Borrow the payload as a str (up to sz bytes), substituting "" on non-UTF-8.
    pub fn data_str(&self) -> &str {
        let sz = (self.sz as usize).min(IPC_DATA_SIZE);
        std::str::from_utf8(&self.data[..sz]).unwrap_or("")
    }

    /// Write the struct to a writer using field-by-field I/O (no unsafe).
    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&self.msg_type.to_ne_bytes())?;
        w.write_all(&self.timeout.to_ne_bytes())?;
        w.write_all(&self.data)?;
        w.write_all(&self.sz.to_ne_bytes())
    }

    /// Read a complete struct from a reader using field-by-field I/O (no unsafe).
    pub fn read_from(r: &mut dyn Read) -> io::Result<Self> {
        let mut msg_type = [0u8; 4];
        let mut timeout  = [0u8; 4];
        let mut data     = [0u8; IPC_DATA_SIZE];
        let mut sz       = [0u8; 8];
        r.read_exact(&mut msg_type)?;
        r.read_exact(&mut timeout)?;
        r.read_exact(&mut data)?;
        r.read_exact(&mut sz)?;
        Ok(Self {
            msg_type: u32::from_ne_bytes(msg_type),
            timeout:  u32::from_ne_bytes(timeout),
            data,
            sz: u64::from_ne_bytes(sz),
        })
    }
}

/// IPC server endpoint. Holds an exclusive lock for the lifetime of the daemon
/// so a second instance fails to start.
#[cfg(unix)]
pub struct IpcServer {
    listener: std::os::unix::net::UnixListener,
    /// Keeps the flock alive; dropping it releases the lock.
    _lock: std::fs::File,
}

#[cfg(unix)]
impl IpcServer {
    pub fn create() -> io::Result<Self> {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        use std::os::unix::io::AsRawFd;

        let lock_path = format!("{SOCKET_PATH}.lock");
        let lock_file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)?;

        // Non-blocking exclusive lock — fails immediately if another daemon owns it.
        // SAFETY: lock_file is an open file descriptor valid for the duration of this call.
        let rc = unsafe { libc::flock(lock_file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        if rc != 0 {
            return Err(io::Error::last_os_error());
        }

        let _ = fs::remove_file(SOCKET_PATH);
        let listener = std::os::unix::net::UnixListener::bind(SOCKET_PATH)?;
        let mut perms = fs::metadata(SOCKET_PATH)?.permissions();
        perms.set_mode(0o660);
        fs::set_permissions(SOCKET_PATH, perms)?;

        Ok(Self { listener, _lock: lock_file })
    }

    /// Raw fd of the listening socket, for the daemon's poll loop.
    pub fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        use std::os::unix::io::AsRawFd;
        self.listener.as_raw_fd()
    }

    /// Accept one pending connection (call only when the fd polls readable).
    pub fn accept(&self) -> io::Result<IpcStream> {
        self.listener.accept().map(|(s, _)| s)
    }
}

/// Connect to the running daemon's IPC endpoint.
#[cfg(unix)]
pub fn ipc_connect() -> io::Result<IpcStream> {
    std::os::unix::net::UnixStream::connect(SOCKET_PATH)
}

// ── Windows: named-pipe server ─────────────────────────────────────────────

/// IPC server over the named pipe `\\.\pipe\keydo`. A background thread
/// blocks in ConnectNamedPipe and queues connected clients; the daemon loop
/// drains the queue. `FILE_FLAG_FIRST_PIPE_INSTANCE` on the first instance
/// doubles as the single-daemon lock.
#[cfg(windows)]
pub struct IpcServer {
    pending: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<IpcStream>>>,
}

#[cfg(windows)]
impl IpcServer {
    pub fn create() -> io::Result<Self> {
        // Created synchronously so a second daemon fails fast (ERROR_ACCESS_DENIED).
        let first = windows_pipe::create_instance(true)?;

        let pending = std::sync::Arc::new(std::sync::Mutex::new(
            std::collections::VecDeque::new(),
        ));
        let queue = std::sync::Arc::clone(&pending);
        std::thread::spawn(move || {
            let mut instance = first;
            loop {
                match instance.wait_for_client() {
                    Ok(stream) => {
                        if let Ok(mut q) = queue.lock() {
                            q.push_back(stream);
                        }
                    }
                    Err(e) => log::warn!("ipc: ConnectNamedPipe failed: {e}"),
                }
                // Each connection consumes one pipe instance; create the next.
                instance = match windows_pipe::create_instance(false) {
                    Ok(h) => h,
                    Err(e) => {
                        log::error!("ipc: CreateNamedPipeW failed: {e}");
                        return;
                    }
                };
            }
        });

        Ok(Self { pending })
    }

    /// Pop one queued client connection, if any (non-blocking).
    pub fn try_accept(&self) -> Option<IpcStream> {
        self.pending.lock().ok()?.pop_front()
    }
}

#[cfg(windows)]
mod windows_pipe {
    use std::io;
    use std::os::windows::io::FromRawHandle;

    use windows_sys::Win32::Foundation::{ERROR_PIPE_CONNECTED, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_FLAG_FIRST_PIPE_INSTANCE, PIPE_ACCESS_DUPLEX,
    };
    use windows_sys::Win32::System::Pipes::{
        ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_TYPE_BYTE,
        PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
    };

    use super::SOCKET_PATH;

    /// Raw pipe-instance handle awaiting a client. Wrapped in a newtype so it
    /// can be sent to the accept thread.
    pub struct PipeInstance(isize);

    // SAFETY: a pipe HANDLE is just a kernel object reference; it is valid on
    // any thread of the creating process.
    unsafe impl Send for PipeInstance {}

    pub fn create_instance(first: bool) -> io::Result<PipeInstance> {
        let mut path: Vec<u16> = SOCKET_PATH.encode_utf16().collect();
        path.push(0);

        let mut open_mode = PIPE_ACCESS_DUPLEX;
        if first {
            open_mode |= FILE_FLAG_FIRST_PIPE_INSTANCE;
        }

        // Byte mode (not message mode) so read_exact sees the same stream
        // semantics as a Unix socket. Default security: same-user access.
        // SAFETY: path is a valid NUL-terminated UTF-16 string; all numeric
        // arguments are documented CreateNamedPipeW values.
        let handle = unsafe {
            CreateNamedPipeW(
                path.as_ptr(),
                open_mode,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                PIPE_UNLIMITED_INSTANCES,
                8192,
                8192,
                0,
                std::ptr::null(),
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        Ok(PipeInstance(handle as isize))
    }

    impl PipeInstance {
        /// Block until a client connects to this instance, then hand the
        /// handle to a `File` (whose drop closes it, disconnecting the client).
        pub fn wait_for_client(self) -> io::Result<super::IpcStream> {
            let handle = self.0 as *mut core::ffi::c_void;
            // SAFETY: handle is the open pipe instance created above; a null
            // overlapped pointer selects blocking mode.
            let ok = unsafe { ConnectNamedPipe(handle, std::ptr::null_mut()) };
            if ok == 0 {
                let err = io::Error::last_os_error();
                // A client that connected between create and connect reports
                // ERROR_PIPE_CONNECTED — that is success.
                if err.raw_os_error() != Some(ERROR_PIPE_CONNECTED as i32) {
                    // SAFETY: handle is owned by this function on the error path.
                    unsafe { windows_sys::Win32::Foundation::CloseHandle(handle) };
                    return Err(err);
                }
            }
            // SAFETY: handle is an open, connected pipe instance; ownership
            // transfers to the File, whose drop closes it exactly once.
            Ok(unsafe { std::fs::File::from_raw_handle(handle) })
        }
    }
}

/// Connect to the daemon's named pipe. `File` open with read+write on a pipe
/// path yields a duplex pipe client.
#[cfg(windows)]
pub fn ipc_connect() -> io::Result<IpcStream> {
    const ERROR_PIPE_BUSY: i32 = 231;
    let open = || std::fs::OpenOptions::new().read(true).write(true).open(SOCKET_PATH);
    // All instances busy: the server is creating the next one; retry briefly.
    for _ in 0..50 {
        match open() {
            Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY) => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            result => return result,
        }
    }
    open()
}

/// Send one IPC message and return the daemon's response data string,
/// or an error if the transport fails or the server responds with Fail.
pub fn ipc_send_recv(
    msg_type: IpcMessageType,
    data: &[u8],
    timeout: u32,
) -> Result<String, KeydoError> {
    let mut stream = ipc_connect().map_err(KeydoError::IpcTransport)?;

    let mut msg = IpcMessage::new(msg_type, timeout);
    msg.set_data(data);
    msg.write_to(&mut stream).map_err(KeydoError::IpcTransport)?;

    let resp = IpcMessage::read_from(&mut stream).map_err(KeydoError::IpcTransport)?;
    let body = resp.data_str().to_string();
    match IpcMessageType::try_from(resp.msg_type) {
        Ok(IpcMessageType::Success) => Ok(body),
        _ => Err(KeydoError::IpcRemoteFailure(body)),
    }
}
