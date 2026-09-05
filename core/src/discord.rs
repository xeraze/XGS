use std::os::windows::ffi::OsStrExt;

#[derive(Debug, Clone)]
pub struct DiscordPresence {
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
        let pipe_name: Vec<u16> = std::ffi::OsStr::new("\\\\.\\pipe\\discord-ipc-0")
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

        if handle == INVALID_HANDLE_VALUE || handle == INVALID_HANDLE {
            return Err(format!(
                "Failed to connect to Discord IPC pipe, error: {}",
                unsafe { GetLastError() }
            ));
        }

        self.pipe_handle = handle;
        self.connected = true;
        Ok(())
    }

    pub fn send_handshake(&mut self) -> Result<(), String> {
        let handshake = serde_json::json!({
            "v": 1,
            "client_id": self.app_id
        });

        self.send_message(0, handshake.to_string())
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

        self.send_message(1, payload.to_string())
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

        self.send_message(1, payload.to_string())
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
                return Err("Failed to write header".to_string());
            }

            let r2 = WriteFile(
                self.pipe_handle,
                bytes.as_ptr(),
                bytes.len() as u32,
                &mut bytes_written,
                std::ptr::null_mut(),
            );
            if r2 == 0 {
                return Err("Failed to write payload".to_string());
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
