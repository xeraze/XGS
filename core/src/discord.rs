use std::os::windows::ffi::OsStrExt;

#[derive(Debug, Clone)]
pub struct DiscordPresence {
    pub name: String,
    pub details: String,
    pub state: String,
    pub large_image: Option<String>,
    pub large_image_text: Option<String>,
    pub small_image: Option<String>,
    pub small_image_text: Option<String>,
    pub start_timestamp: Option<i64>,
}

type HANDLE = *mut core::ffi::c_void;
const INVALID_HANDLE: HANDLE = -1isize as HANDLE;
const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;

extern "system" {
    fn CreateFileW(
        lpfilename: *const u16,
        dwdesiredaccess: u32,
        dwsharemode: u32,
        lpsecurityattributes: *mut core::ffi::c_void,
        dwcreationdisposition: u32,
        dwflagsandattributes: u32,
        htemplatefile: HANDLE,
    ) -> HANDLE;

    fn WriteFile(
        hfile: HANDLE,
        lpbuffer: *const u8,
        nnumberofbytestowrite: u32,
        lpnumberofbyteswritten: *mut u32,
        lpoverlapped: *mut core::ffi::c_void,
    ) -> i32;

    fn ReadFile(
        hfile: HANDLE,
        lpbuffer: *mut u8,
        nnumberofbytestoread: u32,
        lpnumberofbytesread: *mut u32,
        lpoverlapped: *mut core::ffi::c_void,
    ) -> i32;

    fn CloseHandle(hobject: HANDLE) -> i32;

    fn GetLastError() -> u32;
}

const GENERIC_READ: u32 = 0x80000000;
const GENERIC_WRITE: u32 = 0x40000000;
const OPEN_EXISTING: u32 = 3;
const FILE_ATTRIBUTE_NORMAL: u32 = 128;

pub struct DiscordIpc {
    app_id: String,
    connected: bool,
    pipe_handle: HANDLE,
}

unsafe impl Send for DiscordIpc {}
unsafe impl Sync for DiscordIpc {}

impl DiscordIpc {
    pub fn new(app_id: &str) -> Self {
        DiscordIpc {
            app_id: app_id.to_string(),
            connected: false,
            pipe_handle: 0 as HANDLE,
        }
    }

    pub fn connect(&mut self) -> Result<(), String> {
        for idx in 0..10 {
            let pipe_name: Vec<u16> = std::ffi::OsStr::new(&format!("\\\\.\\pipe\\discord-ipc-{}", idx))
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            let handle = unsafe {
                CreateFileW(
                    pipe_name.as_ptr(),
                    GENERIC_READ | GENERIC_WRITE,
                    0,
                    std::ptr::null_mut(),
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL,
                    0 as HANDLE,
                )
            };

            if handle != INVALID_HANDLE_VALUE && handle != INVALID_HANDLE {
                self.pipe_handle = handle;
                self.connected = true;
                log::debug!("Connected to Discord IPC pipe {}", idx);
                std::thread::sleep(std::time::Duration::from_millis(100));
                return Ok(());
            }
        }

        Err(format!(
            "Failed to connect to any Discord IPC pipe, error: {}",
            unsafe { GetLastError() }
        ))
    }

    pub fn send_handshake(&mut self) -> Result<(), String> {
        let handshake = serde_json::json!({
            "v": 1,
            "client_id": self.app_id
        });

        let msg = handshake.to_string();
        log::debug!("Sending handshake ({} bytes): {}", msg.len(), msg);
        self.send_message(0, msg)?;
        log::debug!("Handshake sent, waiting for response...");

        let response = self.read_message()?;
        log::debug!("Handshake response ({} bytes): {}", response.len(), response);

        let parsed: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| format!("Invalid handshake response: {}", e))?;

        if let Some(code) = parsed.get("code") {
            if code.as_i64() != Some(0) {
                let msg = parsed.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown error");
                return Err(format!("Handshake rejected (code {}): {}", code, msg));
            }
        }

        log::info!("Handshake OK");
        Ok(())
    }

    pub fn set_activity(&mut self, presence: &DiscordPresence) -> Result<(), String> {
        let mut assets = serde_json::json!({});

        if let Some(ref img) = presence.large_image {
            assets["large_image"] = serde_json::json!(img);
        }
        if let Some(ref text) = presence.large_image_text {
            assets["large_text"] = serde_json::json!(text);
        }
        if let Some(ref img) = presence.small_image {
            assets["small_image"] = serde_json::json!(img);
        }
        if let Some(ref text) = presence.small_image_text {
            assets["small_text"] = serde_json::json!(text);
        }

        let mut activity = serde_json::json!({
            "name": presence.name,
            "details": presence.details,
            "state": presence.state,
        });

        if !assets.as_object().unwrap().is_empty() {
            activity["assets"] = assets;
        }

        if let Some(start) = presence.start_timestamp {
            activity["timestamps"] = serde_json::json!({
                "start": start
            });
        }

        let payload = serde_json::json!({
            "cmd": "SET_ACTIVITY",
            "args": {
                "pid": std::process::id(),
                "activity": activity
            },
            "nonce": "1"
        });

        match self.send_message(1, payload.to_string()) {
            Ok(()) => {
                match self.read_message() {
                    Ok(resp) => {
                        log::debug!("Activity response: {}", resp);
                        Ok(())
                    }
                    Err(e) => {
                        log::warn!("Failed to read activity response: {}", e);
                        Ok(())
                    }
                }
            }
            Err(e) => {
                log::warn!("Send failed, reconnecting: {}", e);
                self.reconnect()?;
                self.send_message(1, payload.to_string())?;
                match self.read_message() {
                    Ok(resp) => {
                        log::debug!("Activity response after reconnect: {}", resp);
                        Ok(())
                    }
                    Err(e) => {
                        log::warn!("Failed to read response after reconnect: {}", e);
                        Ok(())
                    }
                }
            }
        }
    }

    pub fn clear_activity(&mut self) -> Result<(), String> {
        let payload = serde_json::json!({
            "cmd": "SET_ACTIVITY",
            "args": {
                "pid": std::process::id(),
                "activity": serde_json::Value::Null
            },
            "nonce": "2"
        });

        self.send_message(1, payload.to_string())?;
        let _ = self.read_message();
        Ok(())
    }

    fn reconnect(&mut self) -> Result<(), String> {
        self.connected = false;
        if self.pipe_handle != 0 as HANDLE && self.pipe_handle != INVALID_HANDLE_VALUE {
            unsafe { CloseHandle(self.pipe_handle); }
            self.pipe_handle = 0 as HANDLE;
        }

        std::thread::sleep(std::time::Duration::from_millis(500));

        self.connect()?;
        self.send_handshake()?;
        Ok(())
    }

    fn read_message(&mut self) -> Result<String, String> {
        if !self.connected {
            return Err("Not connected to Discord IPC".to_string());
        }

        let handle_val = self.pipe_handle as usize;
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let handle = handle_val as HANDLE;
            unsafe {
                let mut header = [0u8; 8];
                let mut bytes_read: u32 = 0;
                let r = ReadFile(
                    handle,
                    header.as_mut_ptr(),
                    8,
                    &mut bytes_read,
                    std::ptr::null_mut(),
                );
                if r == 0 || bytes_read < 8 {
                    let _ = tx.send(Err(format!("Failed to read header, error: {}, bytes: {}", GetLastError(), bytes_read)));
                    return;
                }

                let opcode = header[0] as u32
                    | (header[1] as u32) << 8
                    | (header[2] as u32) << 16
                    | (header[3] as u32) << 24;
                let length = header[4] as u32
                    | (header[5] as u32) << 8
                    | (header[6] as u32) << 16
                    | (header[7] as u32) << 24;

                log::debug!("Read message: opcode={}, length={}", opcode, length);

                if length > 65536 {
                    let _ = tx.send(Err(format!("Message too large: {} bytes", length)));
                    return;
                }

                let mut payload = vec![0u8; length as usize];
                let mut total_read = 0u32;

                while total_read < length {
                    let mut chunk = [0u8; 4096];
                    let to_read = std::cmp::min(4096, length - total_read);
                    let mut n: u32 = 0;
                    let r = ReadFile(
                        handle,
                        chunk.as_mut_ptr(),
                        to_read,
                        &mut n,
                        std::ptr::null_mut(),
                    );
                    if r == 0 || n == 0 {
                        let _ = tx.send(Err(format!("Failed to read payload, error: {}", GetLastError())));
                        return;
                    }
                    payload[total_read as usize..(total_read + n) as usize].copy_from_slice(&chunk[..n as usize]);
                    total_read += n;
                }

                let _ = tx.send(String::from_utf8(payload).map_err(|e| format!("Invalid UTF-8: {}", e)));
            }
        });

        match rx.recv_timeout(std::time::Duration::from_secs(5)) {
            Ok(result) => result,
            Err(_) => Err("Timeout reading Discord response".to_string()),
        }
    }

    fn send_message(&mut self, opcode: i32, payload: String) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected to Discord IPC".to_string());
        }

        let bytes = payload.as_bytes();
        let header: [u8; 8] = [
            (opcode & 0xFF) as u8,
            ((opcode >> 8) & 0xFF) as u8,
            ((opcode >> 16) & 0xFF) as u8,
            ((opcode >> 24) & 0xFF) as u8,
            (bytes.len() & 0xFF) as u8,
            ((bytes.len() >> 8) & 0xFF) as u8,
            ((bytes.len() >> 16) & 0xFF) as u8,
            ((bytes.len() >> 24) & 0xFF) as u8,
        ];

        let mut bytes_written: u32 = 0;
        unsafe {
            let r1 = WriteFile(
                self.pipe_handle,
                header.as_ptr(),
                header.len() as u32,
                &mut bytes_written,
                std::ptr::null_mut(),
            );
            if r1 == 0 {
                return Err(format!("Failed to write header, error: {}", GetLastError()));
            }

            let r2 = WriteFile(
                self.pipe_handle,
                bytes.as_ptr(),
                bytes.len() as u32,
                &mut bytes_written,
                std::ptr::null_mut(),
            );
            if r2 == 0 {
                return Err(format!("Failed to write payload, error: {}", GetLastError()));
            }
        }

        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }
}

impl Drop for DiscordIpc {
    fn drop(&mut self) {
        if self.pipe_handle != 0 as HANDLE && self.pipe_handle != INVALID_HANDLE_VALUE {
            unsafe {
                CloseHandle(self.pipe_handle);
            }
        }
    }
}
