use std::os::unix::net::{UnixListener, UnixStream};
use std::io::{Read, Write};
use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcMessageType {
    Success,
    Fail,
    Bind,
    Input,
    Macro,
    Reload,
    LayerListen,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpcMessage {
    pub message_type: IpcMessageType,
    pub timeout: u32,
    pub data: String,
}

pub const SOCKET_PATH: &str = "/var/run/keyd.socket";

pub fn create_server() -> anyhow::Result<UnixListener> {
    if Path::new(SOCKET_PATH).exists() {
        std::fs::remove_file(SOCKET_PATH)?;
    }
    let listener = UnixListener::bind(SOCKET_PATH)?;
    // Set permissions similar to C code
    let mut perms = std::fs::metadata(SOCKET_PATH)?.permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o660);
    std::fs::set_permissions(SOCKET_PATH, perms)?;
    
    Ok(listener)
}

pub fn connect() -> anyhow::Result<UnixStream> {
    Ok(UnixStream::connect(SOCKET_PATH)?)
}
