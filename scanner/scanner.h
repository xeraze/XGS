#pragma once

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>

typedef struct ScannerHandle ScannerHandle;

ScannerHandle* scanner_open_process(const char* process_name);
void scanner_close_process(ScannerHandle* handle);

bool scanner_read_int(ScannerHandle* handle, uintptr_t address, int32_t* out_value);
bool scanner_read_uint(ScannerHandle* handle, uintptr_t address, uint32_t* out_value);
bool scanner_read_float(ScannerHandle* handle, uintptr_t address, float* out_value);
bool scanner_read_string(ScannerHandle* handle, uintptr_t address, char* buffer, int buffer_size);

bool scanner_resolve_pointer_chain(ScannerHandle* handle, uintptr_t base_address,
                                   const uint32_t* offsets, int offset_count,
                                   uintptr_t* out_resolved_address);

uintptr_t scanner_find_module_base(ScannerHandle* handle, const char* module_name);

uintptr_t scanner_aob_scan(ScannerHandle* handle, const uint8_t* pattern,
                           const char* mask, uintptr_t start_address, size_t scan_size);

bool scanner_read_by_config(ScannerHandle* handle,
                            const char* module_name,
                            uintptr_t base_offset,
                            const uint32_t* offsets, int offset_count,
                            int value_type,
                            void* out_value);

#ifdef __cplusplus
}
#endif
