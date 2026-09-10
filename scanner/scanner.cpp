#include "scanner.h"
#include <windows.h>
#include <tlhelp32.h>
#include <psapi.h>
#include <string>
#include <vector>
#include <cstring>

struct ScannerHandle {
    HANDLE process_handle;
    DWORD process_id;
    std::string process_name;
};

static DWORD find_process_id(const char* name) {
    HANDLE snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if (snapshot == INVALID_HANDLE_VALUE) return 0;

    PROCESSENTRY32W entry;
    entry.dwSize = sizeof(entry);

    int wide_len = MultiByteToWideChar(CP_UTF8, 0, name, -1, NULL, 0);
    std::vector<wchar_t> wide_name(wide_len);
    MultiByteToWideChar(CP_UTF8, 0, name, -1, wide_name.data(), wide_len);

    if (Process32FirstW(snapshot, &entry)) {
        do {
            if (_wcsicmp(entry.szExeFile, wide_name.data()) == 0) {
                CloseHandle(snapshot);
                return entry.th32ProcessID;
            }
        } while (Process32NextW(snapshot, &entry));
    }

    CloseHandle(snapshot);
    return 0;
}

extern "C" {

ScannerHandle* scanner_open_process(const char* process_name) {
    DWORD pid = find_process_id(process_name);
    if (pid == 0) return nullptr;

    HANDLE handle = OpenProcess(
        PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
        FALSE, pid
    );
    if (!handle) return nullptr;

    ScannerHandle* scanner = new ScannerHandle;
    scanner->process_handle = handle;
    scanner->process_id = pid;
    scanner->process_name = process_name;
    return scanner;
}

void scanner_close_process(ScannerHandle* handle) {
    if (handle) {
        if (handle->process_handle) {
            CloseHandle(handle->process_handle);
        }
        delete handle;
    }
}

bool scanner_read_int(ScannerHandle* handle, uintptr_t address, int32_t* out_value) {
    if (!handle || !handle->process_handle || !out_value) return false;
    SIZE_T bytes_read = 0;
    return ReadProcessMemory(handle->process_handle, (LPCVOID)address,
                            out_value, sizeof(int32_t), &bytes_read) && bytes_read == sizeof(int32_t);
}

bool scanner_read_uint(ScannerHandle* handle, uintptr_t address, uint32_t* out_value) {
    if (!handle || !handle->process_handle || !out_value) return false;
    SIZE_T bytes_read = 0;
    return ReadProcessMemory(handle->process_handle, (LPCVOID)address,
                            out_value, sizeof(uint32_t), &bytes_read) && bytes_read == sizeof(uint32_t);
}

bool scanner_read_float(ScannerHandle* handle, uintptr_t address, float* out_value) {
    if (!handle || !handle->process_handle || !out_value) return false;
    SIZE_T bytes_read = 0;
    return ReadProcessMemory(handle->process_handle, (LPCVOID)address,
                            out_value, sizeof(float), &bytes_read) && bytes_read == sizeof(float);
}

bool scanner_read_string(ScannerHandle* handle, uintptr_t address, char* buffer, int buffer_size) {
    if (!handle || !handle->process_handle || !buffer || buffer_size <= 1) return false;
    SIZE_T bytes_read = 0;
    if (!ReadProcessMemory(handle->process_handle, (LPCVOID)address,
                          buffer, buffer_size - 1, &bytes_read)) {
        return false;
    }
    buffer[bytes_read < (SIZE_T)(buffer_size - 1) ? bytes_read : buffer_size - 1] = '\0';
    return true;
}

uintptr_t scanner_find_module_base(ScannerHandle* handle, const char* module_name) {
    if (!handle || !handle->process_handle || !module_name) return 0;

    DWORD cb_needed = 0;

    // Сначала узнаём размер, потом выделяем ровно столько, сколько нужно
    // (в процессе может быть больше 1024 модулей).
    if (!EnumProcessModulesEx(handle->process_handle, NULL, 0, &cb_needed, 0)) {
        return 0;
    }
    if (cb_needed == 0) return 0;

    std::vector<HMODULE> modules(cb_needed / sizeof(HMODULE));

    if (!EnumProcessModulesEx(handle->process_handle, modules.data(),
                              cb_needed, &cb_needed, 0)) {
        return 0;
    }

    int count = (int)(cb_needed / sizeof(HMODULE));
    char module_path[MAX_PATH];

    for (int i = 0; i < count; i++) {
        if (GetModuleFileNameExA(handle->process_handle, modules[i], module_path, MAX_PATH)) {
            const char* fname = strrchr(module_path, '\\');
            if (fname && _stricmp(fname + 1, module_name) == 0) {
                return (uintptr_t)modules[i];
            }
        }
    }

    return 0;
}

bool scanner_resolve_pointer_chain(ScannerHandle* handle, uintptr_t base_address,
                                   const uint32_t* offsets, int offset_count,
                                   uintptr_t* out_resolved_address) {
    if (!handle || !handle->process_handle || !out_resolved_address) return false;
    if (offset_count > 0 && !offsets) return false;

    uintptr_t current = base_address;

    for (int i = 0; i < offset_count; i++) {
        uintptr_t ptr = 0;
        SIZE_T bytes_read = 0;
        if (!ReadProcessMemory(handle->process_handle, (LPCVOID)current,
                              &ptr, sizeof(uintptr_t), &bytes_read)) {
            return false;
        }
        current = ptr + offsets[i];
    }

    *out_resolved_address = current;
    return true;
}

uintptr_t scanner_aob_scan(ScannerHandle* handle, const uint8_t* pattern,
                           const char* mask, uintptr_t start_address, size_t scan_size) {
    if (!handle || !handle->process_handle || !mask || !pattern) return 0;

    size_t pattern_len = strlen(mask);
    if (pattern_len == 0) return 0;

    std::vector<uint8_t> buffer(scan_size);
    SIZE_T bytes_read = 0;

    if (!ReadProcessMemory(handle->process_handle, (LPCVOID)start_address,
                          buffer.data(), scan_size, &bytes_read)) {
        return 0;
    }

    if (bytes_read < pattern_len) return 0;

    for (size_t i = 0; i <= bytes_read - pattern_len; i++) {
        bool found = true;
        for (size_t j = 0; j < pattern_len; j++) {
            if (mask[j] != '?' && buffer[i + j] != pattern[j]) {
                found = false;
                break;
            }
        }
        if (found) {
            return start_address + i;
        }
    }

    return 0;
}

bool scanner_read_by_config(ScannerHandle* handle,
                            const char* module_name,
                            uintptr_t base_offset,
                            const uint32_t* offsets, int offset_count,
                            int value_type,
                            void* out_value) {
    if (!handle || !handle->process_handle || !out_value) return false;

    uintptr_t module_base = scanner_find_module_base(handle, module_name);
    if (module_base == 0) return false;

    if (offset_count == 0) {
        uintptr_t address = module_base + base_offset;
        switch (value_type) {
            case 0: return scanner_read_int(handle, address, (int32_t*)out_value);
            case 1: return scanner_read_uint(handle, address, (uint32_t*)out_value);
            case 2: return scanner_read_float(handle, address, (float*)out_value);
            case 3: return scanner_read_string(handle, address, (char*)out_value, 1024);
            default: return false;
        }
    }

    uintptr_t resolved = 0;
    uintptr_t base_addr = module_base + base_offset;

    if (!scanner_resolve_pointer_chain(handle, base_addr, offsets, offset_count - 1, &resolved)) {
        return false;
    }

    uintptr_t final_addr = resolved + offsets[offset_count - 1];

    switch (value_type) {
        case 0: return scanner_read_int(handle, final_addr, (int32_t*)out_value);
        case 1: return scanner_read_uint(handle, final_addr, (uint32_t*)out_value);
        case 2: return scanner_read_float(handle, final_addr, (float*)out_value);
        case 3: return scanner_read_string(handle, final_addr, (char*)out_value, 1024);
        default: return false;
    }
}

bool scanner_read_memory(ScannerHandle* handle, uintptr_t address, uint8_t* buffer, size_t size) {
    if (!handle || !handle->process_handle || !buffer) return false;

    SIZE_T bytes_read = 0;
    BOOL success = ReadProcessMemory(
        handle->process_handle,
        (LPCVOID)address,
        buffer,
        size,
        &bytes_read
    );

    return success && bytes_read > 0;
}

}