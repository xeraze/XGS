#ifndef GAMEPRESENCE_HOOK_H
#define GAMEPRESENCE_HOOK_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdbool.h>
#include <stdint.h>
#include <windows.h>

typedef void (*HookCallback)(void *user_data, const char *function_name,
                             uint64_t return_value, const void *args,
                             size_t args_size);

typedef struct HookHandle HookHandle;

HookHandle *hook_create(const char *target_process, const char *target_dll,
                        const char *function_name, HookCallback callback,
                        void *user_data);
void hook_destroy(HookHandle *handle);

bool hook_enable(HookHandle *handle);
bool hook_disable(HookHandle *handle);

bool hook_inject_dll(HANDLE process_handle, const char *dll_path);
bool hook_eject_dll(HANDLE process_handle, const char *dll_name);

uint64_t hook_call_original(HookHandle *handle, const void *args,
                            size_t args_size);

bool hook_write_jmp(void *target_address, void *hook_function,
                    uint8_t *original_bytes);
bool hook_restore_bytes(void *target_address, const uint8_t *original_bytes,
                        size_t size);

void *hook_get_module_base(HANDLE process_handle, const char *module_name);

#ifdef __cplusplus
}
#endif

#endif