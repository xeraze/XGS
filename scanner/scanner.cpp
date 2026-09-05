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

    PROCESSENTRY32 entry;
    entry.dwSize = sizeof(entry);

    if (Process32First(snapshot, &entry)) {
        do {
            if (_stricmp(entry.szExeFile, name) == 0) {
                CloseHandle(snapshot);
                return entry.th32ProcessID;
            }
        } while (Process32Next(snapshot, &entry));
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
    SIZE_T bytes_read = 0;
    return ReadProcessMemory(handle->process_handle, (LPCVOID)address,
                            out_value, sizeof(int32_t), &bytes_read) && bytes_read == sizeof(int32_t);
}

bool scanner_read_uint(ScannerHandle* handle, uintptr_t address, uint32_t* out_value) {
    SIZE_T bytes_read = 0;
    return ReadProcessMemory(handle->process_handle, (LPCVOID)address,
                            out_value, sizeof(uint32_t), &bytes_read) && bytes_read == sizeof(uint32_t);
}

bool scanner_read_float(ScannerHandle* handle, uintptr_t address, float* out_value) {
    SIZE_T bytes_read = 0;
    return ReadProcessMemory(handle->process_handle, (LPCVOID)address,
                            out_value, sizeof(float), &bytes_read) && bytes_read == sizeof(float);
}

bool scanner_read_string(ScannerHandle* handle, uintptr_t address, char* buffer, int buffer_size) {
    SIZE_T bytes_read = 0;
    if (!ReadProcessMemory(handle->process_handle, (LPCVOID)address,
                          buffer, buffer_size - 1, &bytes_read)) {
        return false;
    }
    buffer[bytes_read] = '\0';
    return true;
}

uintptr_t scanner_find_module_base(ScannerHandle* handle, const char* module_name) {
    HMODULE modules[1024];
    DWORD cb_needed;

    if (EnumProcessModulesEx(handle->process_handle, modules, sizeof(modules), &cb_needed, 0)) {
        int count = cb_needed / sizeof(HMODULE);
        char module_path[MAX_PATH];

        for (int i = 0; i < count; i++) {
            if (GetModuleFileNameExA(handle->process_handle, modules[i], module_path, MAX_PATH)) {
                char* fname = strrchr(module_path, '\\');
                if (fname) {
                    fname++;
                    if (_stricmp(fname, module_name) == 0) {
                        return (uintptr_t)modules[i];
                    }
                }
            }
        }
    }
    return 0;
}

bool scanner_resolve_pointer_chain(ScannerHandle* handle, uintptr_t base_address,
                                   const uint32_t* offsets, int offset_count,
                                   uintptr_t* out_resolved_address) {
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
    if (!mask || !pattern) return 0;

    size_t pattern_len = strlen(mask);
    std::vector<uint8_t> buffer(scan_size);
    SIZE_T bytes_read = 0;

    if (!ReadProcessMemory(handle->process_handle, (LPCVOID)start_address,
                          buffer.data(), scan_size, &bytes_read)) {
        return 0;
    }

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
    uintptr_t module_base = scanner_find_module_base(handle, module_name);
    if (module_base == 0) return false;

    if (offset_count == 0) {
        uintptr_t address = module_base + base_offset;
        switch (value_type) {
            case 0: return scanner_read_int(handle, address, (int32_t*)out_value);
            case 1: return scanner_read_uint(handle, address, (uint32_t*)out_value);
            case 2: return scanner_read_float(handle, address, (float*)out_value);
            case 3: return scanner_read_string(handle, address, (char*)out_value, 256);
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
        case 3: return scanner_read_string(handle, final_addr, (char*)out_value, 256);
        default: return false;
    }
}

} // extern "C"
