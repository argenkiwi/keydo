//! IPC protocol between `keyd` daemon and client subcommands (bind, macro, reload, listen).

use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::io::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};

use crate::error::KeydoError;

pub const SOCKET_PATH: &str = "/var/run/keyd.socket";

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

/// Create the IPC server socket, holding an exclusive lock file to prevent
/// duplicate daemons.  Returns `(listener, lock_file)` — caller must keep
/// `lock_file` alive for the lifetime of the server.
pub fn ipc_create_server() -> io::Result<(UnixListener, fs::File)> {
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
    let listener = UnixListener::bind(SOCKET_PATH)?;
    let mut perms = fs::metadata(SOCKET_PATH)?.permissions();
    perms.set_mode(0o660);
    fs::set_permissions(SOCKET_PATH, perms)?;

    Ok((listener, lock_file))
}

/// Connect to the running daemon's Unix socket.
pub fn ipc_connect() -> io::Result<UnixStream> {
    UnixStream::connect(SOCKET_PATH)
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
