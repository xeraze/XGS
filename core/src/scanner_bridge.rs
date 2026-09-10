use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[repr(C)]
pub struct ScannerHandle {
    _private: [u8; 0],
}

extern "C" {
    fn scanner_open_process(process_name: *const c_char) -> *mut ScannerHandle;
    fn scanner_close_process(handle: *mut ScannerHandle);
    fn scanner_read_int(handle: *mut ScannerHandle, address: usize, out: *mut i32) -> bool;
    fn scanner_read_uint(handle: *mut ScannerHandle, address: usize, out: *mut u32) -> bool;
    fn scanner_read_float(handle: *mut ScannerHandle, address: usize, out: *mut f32) -> bool;
    fn scanner_read_string(
        handle: *mut ScannerHandle,
        address: usize,
        buffer: *mut c_char,
        size: i32,
    ) -> bool;
    fn scanner_find_module_base(handle: *mut ScannerHandle, module_name: *const c_char) -> usize;
    fn scanner_resolve_pointer_chain(
        handle: *mut ScannerHandle,
        base_address: usize,
        offsets: *const u32,
        offset_count: i32,
        out: *mut usize,
    ) -> bool;
    fn scanner_read_by_config(
        handle: *mut ScannerHandle,
        module_name: *const c_char,
        base_offset: usize,
        offsets: *const u32,
        offset_count: i32,
        value_type: i32,
        out_value: *mut std::ffi::c_void,
    ) -> bool;
    fn scanner_aob_scan(
        handle: *mut ScannerHandle,
        pattern: *const u8,
        mask: *const c_char,
        start_address: usize,
        scan_size: usize,
    ) -> usize;
    fn scanner_read_memory(
        handle: *mut ScannerHandle,
        address: usize,
        buffer: *mut u8,
        size: usize,
    ) -> bool;
}

pub struct Scanner {
    handle: *mut ScannerHandle,
}

// SAFETY: Scanner wraps a Windows process HANDLE obtained via OpenProcess.
// Windows handles are thread-safe for synchronous ReadProcessMemory calls.
// The C++ scanner functions use only synchronous I/O and do not share mutable state.
unsafe impl Send for Scanner {}
unsafe impl Sync for Scanner {}

impl Scanner {
    pub fn open(process_name: &str) -> Option<Self> {
        let c_name = CString::new(process_name).ok()?;
        let handle = unsafe { scanner_open_process(c_name.as_ptr()) };
        if handle.is_null() {
            None
        } else {
            Some(Scanner { handle })
        }
    }

    pub fn read_int(&self, address: usize) -> Option<i32> {
        let mut value: i32 = 0;
        if unsafe { scanner_read_int(self.handle, address, &mut value) } {
            Some(value)
        } else {
            None
        }
    }

    pub fn read_uint(&self, address: usize) -> Option<u32> {
        let mut value: u32 = 0;
        if unsafe { scanner_read_uint(self.handle, address, &mut value) } {
            Some(value)
        } else {
            None
        }
    }

    pub fn read_float(&self, address: usize) -> Option<f32> {
        let mut value: f32 = 0.0;
        if unsafe { scanner_read_float(self.handle, address, &mut value) } {
            Some(value)
        } else {
            None
        }
    }

    pub fn read_string(&self, address: usize, buffer_size: usize) -> Option<String> {
        let mut buffer = vec![0i8; buffer_size.min(i32::MAX as usize)];
        if unsafe {
            scanner_read_string(
                self.handle,
                address,
                buffer.as_mut_ptr(),
                buffer.len() as i32,
            )
        } {
            let cstr = unsafe { CStr::from_ptr(buffer.as_ptr()) };
            cstr.to_str().ok().map(|s| s.to_string())
        } else {
            None
        }
    }

    pub fn read_memory(&self, address: usize, buffer: &mut [u8]) -> bool {
        unsafe {
            scanner_read_memory(
                self.handle,
                address,
                buffer.as_mut_ptr(),
                buffer.len(),
            )
        }
    }

    pub fn find_module_base(&self, module_name: &str) -> Option<usize> {
        let c_name = CString::new(module_name).ok()?;
        let base = unsafe { scanner_find_module_base(self.handle, c_name.as_ptr()) };
        if base != 0 {
            Some(base)
        } else {
            None
        }
    }

    pub fn resolve_pointer_chain(&self, base_address: usize, offsets: &[u32]) -> Option<usize> {
        let mut resolved: usize = 0;
        if unsafe {
            scanner_resolve_pointer_chain(
                self.handle,
                base_address,
                offsets.as_ptr(),
                offsets.len() as i32,
                &mut resolved,
            )
        } {
            Some(resolved)
        } else {
            None
        }
    }

    pub fn read_by_config(
        &self,
        module_name: &str,
        base_offset: usize,
        offsets: &[u32],
        value_type: i32,
    ) -> Option<Box<dyn std::any::Any>> {
        let c_name = CString::new(module_name).ok()?;
        let mut int_val: i32 = 0;
        let mut uint_val: u32 = 0;
        let mut float_val: f32 = 0.0;
        let mut str_val = [0i8; 1024];

        let out_ptr = match value_type {
            0 => &mut int_val as *mut i32 as *mut std::ffi::c_void,
            1 => &mut uint_val as *mut u32 as *mut std::ffi::c_void,
            2 => &mut float_val as *mut f32 as *mut std::ffi::c_void,
            3 => str_val.as_mut_ptr() as *mut std::ffi::c_void,
            _ => return None,
        };

        let success = unsafe {
            scanner_read_by_config(
                self.handle,
                c_name.as_ptr(),
                base_offset,
                offsets.as_ptr(),
                offsets.len() as i32,
                value_type,
                out_ptr,
            )
        };

        if !success {
            return None;
        }

        match value_type {
            0 => Some(Box::new(int_val)),
            1 => Some(Box::new(uint_val)),
            2 => Some(Box::new(float_val)),
            3 => {
                let cstr = unsafe { CStr::from_ptr(str_val.as_ptr()) };
                Some(Box::new(cstr.to_str().unwrap_or("").to_string()))
            }
            _ => None,
        }
    }

    pub fn aob_scan(&self, pattern: &[u8], mask: &str, start_address: usize, scan_size: usize) -> Option<usize> {
        // Защита от рассогласования длины маски и паттерна (C++ использует
        // strlen(mask) как длину паттерна). Если длины не совпадают —
        // генерируем точную маску, чтобы не было выхода за границы.
        let c_mask = if mask.len() == pattern.len() {
            CString::new(mask).ok()?
        } else {
            CString::new("x".repeat(pattern.len())).ok()?
        };
        let addr = unsafe {
            scanner_aob_scan(
                self.handle,
                pattern.as_ptr(),
                c_mask.as_ptr(),
                start_address,
                scan_size,
            )
        };
        if addr != 0 {
            Some(addr)
        } else {
            None
        }
    }

    /// AOB-скан без маски: все байты паттерна обязательны.
    /// Маска строится автоматически по длине паттерна, чтобы никогда
    /// не рассогласовать pattern_len (C++ читает mask[j] до strlen(mask)).
    pub fn aob_scan_exact(&self, pattern: &[u8], start_address: usize, scan_size: usize) -> Option<usize> {
        if pattern.is_empty() {
            return None;
        }
        let mask = "x".repeat(pattern.len());
        self.aob_scan(pattern, &mask, start_address, scan_size)
    }

    pub fn find_all_aob(&self, pattern: &[u8], mask: &str, start_address: usize, scan_size: usize) -> Vec<usize> {
        let mut results = Vec::new();
        let mut current_start = start_address;
        let mut remaining = scan_size;

        while remaining > 0 {
            if let Some(addr) = self.aob_scan(pattern, mask, current_start, remaining) {
                // Защита от бесконечного цикла / underflow.
                if addr < current_start {
                    break;
                }
                results.push(addr);
                let found_offset = addr - current_start;
                current_start = addr.saturating_add(1);
                remaining = remaining.saturating_sub(found_offset + 1);
            } else {
                break;
            }
        }

        results
    }

    pub fn scan_for_player_data(&self, process_name: &str, engine: &str) -> Option<PlayerDataOffsets> {
        let base = match self.find_module_base(process_name) {
            Some(b) => {
                log::info!("[XGS] Module base for {}: 0x{:X}", process_name, b);
                b
            }
            None => {
                log::warn!("[XGS] Could not find module base for {}", process_name);
                return None;
            }
        };
        
        let scan_size = 100 * 1024 * 1024;
        log::info!("[XGS] Scanning {} MB of memory starting at 0x{:X}", scan_size / (1024 * 1024), base);
        
        match engine {
            "unity" => self.scan_unity_player_data(base, scan_size),
            "unreal_engine_4" | "unreal_engine_5" => self.scan_unreal_player_data(base, scan_size),
            _ => self.scan_generic_player_data(base, scan_size),
        }
    }

    pub fn scan_for_strings(&self, process_name: &str) -> Vec<String> {
        let base = match self.find_module_base(process_name) {
            Some(b) => b,
            None => return Vec::new(),
        };
        
        let scan_size = 100 * 1024 * 1024;
        let mut found_strings = Vec::new();
        let mut buffer = vec![0u8; 4096];
        let mut current_string = Vec::new();
        
        log::info!("[XGS] Scanning for readable strings in memory...");
        
        for offset in (0..scan_size).step_by(4096) {
            let addr = base + offset;
            if self.read_memory(addr, &mut buffer) {
                for &byte in &buffer {
                    if (byte >= 32 && byte < 127) || (byte >= 0xC0) {
                        current_string.push(byte);
                    } else {
                        if current_string.len() >= 3 && current_string.len() <= 80 {
                            if let Ok(s) = String::from_utf8(current_string.clone()) {
                                let lower = s.to_lowercase();
                                if self.is_meaningful_string(&lower) {
                                    found_strings.push(s);
                                }
                            }
                        }
                        current_string.clear();
                    }
                }
            }
        }
        
        found_strings.sort();
        found_strings.dedup();
        
        log::info!("[XGS] Found {} meaningful strings", found_strings.len());
        found_strings
    }

    fn is_meaningful_string(&self, s: &str) -> bool {
        let keywords = [
            "menu", "lobby", "match", "game", "play", "pause", "score",
            "level", "round", "wave", "mission", "objective", "team",
            "assassin", "tank", "medic", "sniper", "soldier", "engineer",
            "captain", "commander", "scout", "heavy", "support",
            "deathmatch", "teamdeathmatch", "capture", "ctf", "king",
            "survival", "horde", "zombie", "infection", "escape",
            "solo", "duo", "squad", "trio",
            "ranked", "casual", "competitive", "unrated",
            "win", "lose", "draw", "victory", "defeat",
            "kill", "death", "assist", "streak",
            "weapon", "gun", "rifle", "shotgun", "pistol",
            "health", "armor", "shield", "heal", "revive",
            "reload", "ammo", "bullet", "grenade", "ability",
            "spawn", "respawn", "eliminated", "alive",
            "\u{43C}\u{435}\u{43D}\u{44E}",
            "\u{43B}\u{43E}\u{431}\u{431}\u{438}",
            "\u{43C}\u{430}\u{442}\u{447}",
            "\u{438}\u{433}\u{440}\u{430}",
            "\u{43F}\u{430}\u{443}\u{437}\u{430}",
            "\u{441}\u{447}\u{435}\u{442}",
            "\u{443}\u{440}\u{43E}\u{432}\u{435}\u{43D}\u{44C}",
            "\u{43A}\u{43E}\u{43C}\u{430}\u{43D}\u{434}\u{430}",
            "\u{43E}\u{431}\u{44A}\u{435}\u{43A}\u{442}\u{438}\u{432}",
            "\u{43A}\u{43E}\u{43C}\u{430}\u{43D}\u{434}\u{438}\u{440}",
            "\u{441}\u{43D}\u{438}\u{43F}\u{435}\u{440}",
            "\u{442}\u{44F}\u{436}\u{435}\u{43B}\u{44C}",
            "\u{432}\u{43E}\u{438}\u{43D}",
            "\u{432}\u{440}\u{430}\u{433}",
            "\u{441}\u{43A}\u{43E}\u{440}\u{43E}\u{441}\u{442}\u{44C}",
            "\u{43F}\u{43E}\u{431}\u{435}\u{434}\u{430}",
            "\u{43F}\u{43E}\u{440}\u{430}\u{436}\u{435}\u{43D}\u{438}\u{435}",
            "\u{43F}\u{440}\u{43E}\u{432}\u{430}\u{43B}",
            "\u{433}\u{440}\u{43E}\u{43C}\u{43A}\u{438}",
            "\u{441}\u{43F}\u{430}\u{441}\u{442}\u{43D}\u{44B}\u{439}",
            "\u{432}\u{43E}\u{437}\u{440}\u{43E}\u{436}\u{434}\u{435}\u{43D}\u{438}\u{435}",
            "\u{432}\u{44B}\u{436}\u{438}\u{442}\u{44C}",
            "\u{43F}\u{43E}\u{440}\u{430}\u{436}\u{435}\u{43D}\u{438}\u{435}",
            "\u{441}\u{442}\u{430}\u{440}\u{442}",
            "\u{43D}\u{430}\u{447}\u{430}\u{43B}\u{43E}",
            "\u{43A}\u{43E}\u{43D}\u{435}\u{446}",
            "\u{43F}\u{430}\u{443}\u{437}\u{430}",
            "\u{43F}\u{43E}\u{43A}\u{443}\u{43F}\u{43A}\u{438}",
            "\u{43C}\u{430}\u{433}\u{430}\u{437}\u{438}\u{43D}",
            "\u{432}\u{43E}\u{43E}\u{440}\u{443}\u{436}\u{435}\u{43D}\u{438}\u{435}",
            "\u{441}\u{43D}\u{430}\u{440}\u{44F}\u{434}",
            "\u{432}\u{438}\u{434}",
            "\u{438}\u{433}\u{440}\u{43E}\u{43A}",
            "\u{441}\u{442}\u{440}\u{435}\u{43B}\u{43A}\u{438}",
            "\u{437}\u{43E}\u{43B}\u{43E}\u{442}\u{43E}",
            "\u{441}\u{435}\u{440}\u{432}\u{435}\u{440}",
            "\u{43B}\u{435}\u{441}",
            "\u{434}\u{435}\u{440}\u{435}\u{432}\u{43D}\u{44F}",
            "\u{43F}\u{443}\u{441}\u{442}\u{44B}\u{43D}\u{44F}",
            "\u{432}\u{443}\u{43B}\u{43A}\u{430}\u{43D}",
            "\u{431}\u{430}\u{437}\u{430}",
            "\u{441}\u{442}\u{430}\u{43D}\u{446}\u{438}\u{44F}",
            "\u{444}\u{430}\u{431}\u{440}\u{438}\u{43A}\u{430}",
            "\u{437}\u{430}\u{432}\u{43E}\u{434}",
            "\u{445}\u{440}\u{430}\u{43D}\u{438}\u{449}\u{435}",
            "\u{430}\u{440}\u{435}\u{43D}\u{430}",
            "\u{432}\u{43E}\u{43B}\u{43A}\u{430}\u{43D}",
            "\u{445}\u{440}\u{430}\u{43C}",
            "\u{445}\u{440}\u{430}\u{43D}\u{438}\u{449}\u{435}",
            "\u{437}\u{430}\u{43C}\u{43E}\u{43A}",
            "\u{43F}\u{430}\u{442}\u{440}\u{43E}\u{43D}",
            "\u{44F}\u{434}\u{435}\u{440}\u{436}",
            "\u{43F}\u{443}\u{43B}\u{435}\u{43C}\u{435}\u{442}",
            "\u{441}\u{438}\u{43B}\u{430}",
            "\u{441}\u{43A}\u{43E}\u{440}\u{43E}\u{441}\u{442}\u{44C}",
            "\u{437}\u{434}\u{43E}\u{440}\u{43E}\u{432}\u{44C}\u{435}",
            "\u{431}\u{440}\u{43E}\u{43D}\u{44F}",
            "\u{449}\u{438}\u{442}",
            "\u{432}\u{438}\u{434}",
            "\u{43F}\u{43E}\u{440}\u{430}\u{436}\u{435}\u{43D}\u{438}\u{435}",
            "\u{441}\u{442}\u{440}\u{435}\u{43B}\u{43A}\u{438}",
            "\u{43F}\u{43E}\u{431}\u{435}\u{434}\u{430}",
            "\u{43F}\u{43E}\u{440}\u{430}\u{436}\u{435}\u{43D}\u{438}\u{435}",
            "\u{43F}\u{430}\u{443}\u{437}\u{430}",
            "\u{432}\u{43E}\u{43E}\u{440}\u{443}\u{436}\u{435}\u{43D}\u{438}\u{435}",
            "\u{43F}\u{440}\u{43E}\u{43A}\u{43B}\u{430}\u{434}",
            "\u{43F}\u{440}\u{438}\u{446}\u{435}\u{43B}",
            "\u{43C}\u{438}\u{448}\u{435}\u{43D}\u{44C}",
            "\u{43C}\u{430}\u{433}\u{430}\u{437}\u{438}\u{43D}",
            "\u{431}\u{43E}\u{435}\u{432}\u{438}\u{43A}",
            "\u{43A}\u{43B}\u{430}\u{43D}",
            "\u{441}\u{43F}\u{438}\u{441}\u{43E}\u{43A}",
            "\u{43A}\u{43E}\u{43C}\u{430}\u{43D}\u{434}\u{430}",
            "\u{43F}\u{43E}\u{434}\u{440}\u{43E}\u{431}\u{43D}\u{43E}\u{441}\u{442}\u{438}",
            "\u{434}\u{435}\u{439}\u{441}\u{442}\u{432}\u{438}\u{435}",
            "\u{432}\u{44B}\u{43F}\u{43E}\u{43B}\u{43D}\u{438}\u{442}\u{44C}",
            "\u{43F}\u{43E}\u{434}\u{442}\u{432}\u{435}\u{440}\u{436}\u{434}\u{435}\u{43D}\u{438}\u{435}",
            "\u{432}\u{44B}\u{431}\u{43E}\u{440}",
            "\u{43D}\u{430}\u{441}\u{442}\u{440}\u{43E}\u{439}\u{43A}\u{438}",
            "\u{43F}\u{43E}\u{434}\u{442}\u{432}\u{435}\u{440}\u{436}\u{434}\u{435}\u{43D}\u{438}\u{435}",
        ];
        
        for keyword in &keywords {
            if s.contains(keyword) {
                return true;
            }
        }
        
        false
    }

    fn scan_unity_player_data(&self, base: usize, scan_size: usize) -> Option<PlayerDataOffsets> {
        let health_patterns = [
            (vec![0x68, 0x65, 0x61, 0x6C, 0x74, 0x68, 0x00], "health\0"),
            (vec![0x48, 0x50, 0x00], "HP\0"),
            (vec![0x4C, 0x69, 0x66, 0x65, 0x00], "Life\0"),
        ];

        let level_patterns = [
            (vec![0x6C, 0x65, 0x76, 0x65, 0x6C, 0x00], "level\0"),
            (vec![0x4C, 0x65, 0x76, 0x65, 0x6C, 0x00], "Level\0"),
            (vec![0x58, 0x50, 0x00], "XP\0"),
        ];

        let class_patterns = [
            (vec![0x63, 0x6C, 0x61, 0x73, 0x73, 0x00], "class\0"),
            (vec![0x72, 0x6F, 0x6C, 0x65, 0x00], "role\0"),
            (vec![0x74, 0x79, 0x70, 0x65, 0x00], "type\0"),
        ];

        let mut offsets = PlayerDataOffsets::default();

        log::info!("[XGS] Searching for health patterns...");
        for (pattern, name) in &health_patterns {
            if let Some(addr) = self.aob_scan(pattern, &"xxxxxxx", base, scan_size) {
                let relative = addr - base;
                log::info!("[XGS] Found health pattern '{}' at offset 0x{:X}", name, relative);
                offsets.health_offset = Some(relative);
                break;
            }
        }
        if offsets.health_offset.is_none() {
            log::warn!("[XGS] No health pattern found");
        }

        log::info!("[XGS] Searching for level patterns...");
        for (pattern, name) in &level_patterns {
            if let Some(addr) = self.aob_scan(pattern, &"xxxxxx", base, scan_size) {
                let relative = addr - base;
                log::info!("[XGS] Found level pattern '{}' at offset 0x{:X}", name, relative);
                offsets.level_offset = Some(relative);
                break;
            }
        }
        if offsets.level_offset.is_none() {
            log::warn!("[XGS] No level pattern found");
        }

        log::info!("[XGS] Searching for class patterns...");
        for (pattern, name) in &class_patterns {
            if let Some(addr) = self.aob_scan(pattern, &"xxxxx", base, scan_size) {
                let relative = addr - base;
                log::info!("[XGS] Found class pattern '{}' at offset 0x{:X}", name, relative);
                offsets.class_offset = Some(relative);
                break;
            }
        }
        if offsets.class_offset.is_none() {
            log::warn!("[XGS] No class pattern found");
        }

        if offsets.health_offset.is_some() || offsets.level_offset.is_some() {
            Some(offsets)
        } else {
            None
        }
    }

    fn scan_unreal_player_data(&self, base: usize, scan_size: usize) -> Option<PlayerDataOffsets> {
        let health_patterns = [
            (vec![0x48, 0x65, 0x61, 0x6C, 0x74, 0x68, 0x00], "Health\0"),
            (vec![0x48, 0x50, 0x00], "HP\0"),
            (vec![0x43, 0x75, 0x72, 0x72, 0x65, 0x6E, 0x74, 0x48, 0x65, 0x61, 0x6C, 0x74, 0x68, 0x00], "CurrentHealth\0"),
        ];

        let level_patterns = [
            (vec![0x4C, 0x65, 0x76, 0x65, 0x6C, 0x00], "Level\0"),
            (vec![0x50, 0x6C, 0x61, 0x79, 0x65, 0x72, 0x4C, 0x65, 0x76, 0x65, 0x6C, 0x00], "PlayerLevel\0"),
            (vec![0x45, 0x78, 0x70, 0x65, 0x72, 0x69, 0x65, 0x6E, 0x63, 0x65, 0x00], "Experience\0"),
        ];

        let class_patterns = [
            (vec![0x43, 0x6C, 0x61, 0x73, 0x73, 0x00], "Class\0"),
            (vec![0x52, 0x6F, 0x6C, 0x65, 0x00], "Role\0"),
            (vec![0x43, 0x68, 0x61, 0x72, 0x61, 0x63, 0x74, 0x65, 0x72, 0x54, 0x79, 0x70, 0x65, 0x00], "CharacterType\0"),
        ];

        let mut offsets = PlayerDataOffsets::default();

        for (pattern, name) in &health_patterns {
            if let Some(addr) = self.aob_scan(pattern, &"xxxxxxx", base, scan_size) {
                let relative = addr - base;
                log::info!("Found health pattern '{}' at offset 0x{:X}", name, relative);
                offsets.health_offset = Some(relative);
                break;
            }
        }

        for (pattern, name) in &level_patterns {
            if let Some(addr) = self.aob_scan(pattern, &"xxxxxx", base, scan_size) {
                let relative = addr - base;
                log::info!("Found level pattern '{}' at offset 0x{:X}", name, relative);
                offsets.level_offset = Some(relative);
                break;
            }
        }

        for (pattern, name) in &class_patterns {
            if let Some(addr) = self.aob_scan(pattern, &"xxxxx", base, scan_size) {
                let relative = addr - base;
                log::info!("Found class pattern '{}' at offset 0x{:X}", name, relative);
                offsets.class_offset = Some(relative);
                break;
            }
        }

        if offsets.health_offset.is_some() || offsets.level_offset.is_some() {
            Some(offsets)
        } else {
            None
        }
    }

    fn scan_generic_player_data(&self, base: usize, scan_size: usize) -> Option<PlayerDataOffsets> {
        let health_patterns = [
            (vec![0x68, 0x65, 0x61, 0x6C, 0x74, 0x68, 0x00], "health\0"),
            (vec![0x48, 0x50, 0x00], "HP\0"),
            (vec![0x4C, 0x69, 0x66, 0x65, 0x00], "Life\0"),
            (vec![0x56, 0x69, 0x74, 0x61, 0x6C, 0x69, 0x74, 0x79, 0x00], "Vitality\0"),
        ];

        let level_patterns = [
            (vec![0x6C, 0x65, 0x76, 0x65, 0x6C, 0x00], "level\0"),
            (vec![0x4C, 0x65, 0x76, 0x65, 0x6C, 0x00], "Level\0"),
            (vec![0x52, 0x61, 0x6E, 0x6B, 0x00], "Rank\0"),
        ];

        let class_patterns = [
            (vec![0x63, 0x6C, 0x61, 0x73, 0x73, 0x00], "class\0"),
            (vec![0x72, 0x6F, 0x6C, 0x65, 0x00], "role\0"),
            (vec![0x74, 0x79, 0x70, 0x65, 0x00], "type\0"),
        ];

        let mut offsets = PlayerDataOffsets::default();

        for (pattern, name) in &health_patterns {
            if let Some(addr) = self.aob_scan(pattern, &"xxxxxxx", base, scan_size) {
                let relative = addr - base;
                log::info!("Found health pattern '{}' at offset 0x{:X}", name, relative);
                offsets.health_offset = Some(relative);
                break;
            }
        }

        for (pattern, name) in &level_patterns {
            if let Some(addr) = self.aob_scan(pattern, &"xxxxxx", base, scan_size) {
                let relative = addr - base;
                log::info!("Found level pattern '{}' at offset 0x{:X}", name, relative);
                offsets.level_offset = Some(relative);
                break;
            }
        }

        for (pattern, name) in &class_patterns {
            if let Some(addr) = self.aob_scan(pattern, &"xxxxx", base, scan_size) {
                let relative = addr - base;
                log::info!("Found class pattern '{}' at offset 0x{:X}", name, relative);
                offsets.class_offset = Some(relative);
                break;
            }
        }

        if offsets.health_offset.is_some() || offsets.level_offset.is_some() {
            Some(offsets)
        } else {
            None
        }
    }

    pub fn scan_for_game_state(&self, process_name: &str) -> Option<GameStateOffsets> {
        let base = match self.find_module_base(process_name) {
            Some(b) => b,
            None => return None,
        };
        
        let scan_size = 100 * 1024 * 1024;
        let mut offsets = GameStateOffsets::default();
        
        let state_patterns = [
            (vec![0x4D, 0x65, 0x6E, 0x75, 0x00], "Menu\0", "menu"),
            (vec![0x4C, 0x6F, 0x62, 0x62, 0x79, 0x00], "Lobby\0", "lobby"),
            (vec![0x4D, 0x61, 0x74, 0x63, 0x68, 0x00], "Match\0", "match"),
            (vec![0x50, 0x61, 0x75, 0x73, 0x65, 0x00], "Pause\0", "pause"),
        ];
        
        log::info!("[XGS] Scanning for game state strings...");
        for (pattern, name, state) in &state_patterns {
            if let Some(addr) = self.aob_scan(pattern, &"xxxxxxx", base, scan_size) {
                let relative = addr - base;
                log::info!("[XGS] Found state '{}' at offset 0x{:X}", name, relative);
                offsets.state_offsets.insert(state.to_string(), relative);
            }
        }
        
        let class_patterns = [
            (vec![0x41, 0x73, 0x73, 0x61, 0x73, 0x73, 0x69, 0x6E, 0x00], "Assassin\0", "assassin"),
            (vec![0x54, 0x61, 0x6E, 0x6B, 0x00], "Tank\0", "tank"),
            (vec![0x4D, 0x65, 0x64, 0x69, 0x63, 0x00], "Medic\0", "medic"),
            (vec![0x53, 0x6E, 0x69, 0x70, 0x65, 0x72, 0x00], "Sniper\0", "sniper"),
            (vec![0x53, 0x6F, 0x6C, 0x64, 0x69, 0x65, 0x72, 0x00], "Soldier\0", "soldier"),
            (vec![0x45, 0x6E, 0x67, 0x69, 0x6E, 0x65, 0x65, 0x72, 0x00], "Engineer\0", "engineer"),
        ];
        
        log::info!("[XGS] Scanning for class names...");
        for (pattern, name, class) in &class_patterns {
            if let Some(addr) = self.aob_scan(pattern, &"xxxxxxxx", base, scan_size) {
                let relative = addr - base;
                log::info!("[XGS] Found class '{}' at offset 0x{:X}", name, relative);
                offsets.class_offsets.insert(class.to_string(), relative);
            }
        }
        
        Some(offsets)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PlayerDataOffsets {
    pub health_offset: Option<usize>,
    pub level_offset: Option<usize>,
    pub class_offset: Option<usize>,
    pub name_offset: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct GameStateOffsets {
    pub state_offsets: std::collections::HashMap<String, usize>,
    pub class_offsets: std::collections::HashMap<String, usize>,
}

impl Drop for Scanner {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                scanner_close_process(self.handle);
            }
        }
    }
}
