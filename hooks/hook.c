#include "hook.h"
#include <psapi.h>
#include <string.h>
#include <tlhelp32.h>

struct HookHandle {
  HANDLE process_handle;
  DWORD process_id;
  char target_dll[256];
  char function_name[256];
  HookCallback callback;
  void *user_data;
  void *original_address;
  uint8_t original_bytes[16];
  bool is_hooked;
};

static DWORD find_process_by_name(const char *name) {
  PROCESSENTRY32 entry;
  HANDLE snapshot;

  snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
  if (snapshot == INVALID_HANDLE_VALUE)
    return 0;

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

static void *find_module_in_process(HANDLE process, const char *module_name) {
  HMODULE modules[1024];
  DWORD cb_needed;

  if (EnumProcessModulesEx(process, modules, sizeof(modules), &cb_needed, 0)) {
    int count = cb_needed / sizeof(HMODULE);
    char module_path[MAX_PATH];
    int i;

    for (i = 0; i < count; i++) {
      if (GetModuleFileNameExA(process, modules[i], module_path, MAX_PATH)) {
        char *fname = strrchr(module_path, '\\');
        if (fname) {
          fname++;
          if (_stricmp(fname, module_name) == 0) {
            return (void *)modules[i];
          }
        }
      }
    }
  }
  return NULL;
}

static void *find_function_address(HANDLE process, void *module_base,
                                   const char *function_name) {
  HMODULE local_module = NULL;
  char module_path[MAX_PATH];
  FARPROC proc;
  ptrdiff_t offset;
  (void)process;

  if (GetModuleFileNameA((HMODULE)module_base, module_path, MAX_PATH)) {
    local_module = LoadLibraryA(module_path);
  }

  if (!local_module)
    return NULL;

  proc = GetProcAddress(local_module, function_name);
  if (!proc) {
    FreeLibrary(local_module);
    return NULL;
  }

  offset = (char *)(uintptr_t)proc - (char *)local_module;
  FreeLibrary(local_module);

  return (char *)module_base + offset;
}

HookHandle *hook_create(const char *target_process, const char *target_dll,
                        const char *function_name, HookCallback callback,
                        void *user_data) {
  DWORD pid;
  HANDLE handle;
  HookHandle *hook;
  void *module_base;
  SIZE_T bytes_read = 0;

  pid = find_process_by_name(target_process);
  if (pid == 0)
    return NULL;

  handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ |
                           PROCESS_VM_WRITE | PROCESS_VM_OPERATION,
                       FALSE, pid);
  if (!handle)
    return NULL;

  hook = (HookHandle *)calloc(1, sizeof(HookHandle));
  if (!hook) {
    CloseHandle(handle);
    return NULL;
  }

  hook->process_handle = handle;
  hook->process_id = pid;
  strncpy(hook->target_dll, target_dll, sizeof(hook->target_dll) - 1);
  strncpy(hook->function_name, function_name, sizeof(hook->function_name) - 1);
  hook->callback = callback;
  hook->user_data = user_data;
  hook->is_hooked = false;

  module_base = find_module_in_process(handle, target_dll);
  if (!module_base) {
    CloseHandle(handle);
    free(hook);
    return NULL;
  }

  hook->original_address =
      find_function_address(handle, module_base, function_name);
  if (!hook->original_address) {
    CloseHandle(handle);
    free(hook);
    return NULL;
  }

  ReadProcessMemory(handle, hook->original_address, hook->original_bytes,
                    sizeof(hook->original_bytes), &bytes_read);

  return hook;
}

void hook_destroy(HookHandle *handle) {
  if (!handle)
    return;

  if (handle->is_hooked) {
    hook_disable(handle);
  }

  if (handle->process_handle) {
    CloseHandle(handle->process_handle);
  }

  free(handle);
}

bool hook_enable(HookHandle *handle) {
  uint8_t jmp_patch[14];
  uintptr_t addr;
  DWORD old_protect;
  SIZE_T written = 0;

  if (!handle || handle->is_hooked)
    return false;
  if (!handle->original_address)
    return false;

  jmp_patch[0] = 0xFF;
  jmp_patch[1] = 0x25;
  jmp_patch[2] = 0x00;
  jmp_patch[3] = 0x00;
  jmp_patch[4] = 0x00;
  jmp_patch[5] = 0x00;

  addr = (uintptr_t)handle->callback;
  memcpy(&jmp_patch[6], &addr, sizeof(uintptr_t));

  if (!VirtualProtectEx(handle->process_handle, handle->original_address, 14,
                        PAGE_EXECUTE_READWRITE, &old_protect)) {
    return false;
  }

  WriteProcessMemory(handle->process_handle, handle->original_address,
                     jmp_patch, 14, &written);
  VirtualProtectEx(handle->process_handle, handle->original_address, 14,
                   old_protect, &old_protect);

  if (written != 14)
    return false;

  handle->is_hooked = true;
  return true;
}

bool hook_disable(HookHandle *handle) {
  DWORD old_protect;
  SIZE_T written = 0;

  if (!handle || !handle->is_hooked)
    return false;

  if (!VirtualProtectEx(handle->process_handle, handle->original_address, 14,
                        PAGE_EXECUTE_READWRITE, &old_protect)) {
    return false;
  }

  WriteProcessMemory(handle->process_handle, handle->original_address,
                     handle->original_bytes, 14, &written);
  VirtualProtectEx(handle->process_handle, handle->original_address, 14,
                   old_protect, &old_protect);

  if (written != 14)
    return false;

  handle->is_hooked = false;
  return true;
}

bool hook_inject_dll(HANDLE process_handle, const char *dll_path) {
  size_t path_len;
  void *remote_mem;
  SIZE_T written = 0;
  HMODULE kernel32;
  FARPROC load_lib;
  HANDLE thread;

  if (!process_handle || !dll_path)
    return false;

  path_len = strlen(dll_path) + 1;
  remote_mem = VirtualAllocEx(process_handle, NULL, path_len,
                              MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
  if (!remote_mem)
    return false;

  if (!WriteProcessMemory(process_handle, remote_mem, dll_path, path_len,
                          &written)) {
    VirtualFreeEx(process_handle, remote_mem, 0, MEM_RELEASE);
    return false;
  }

  kernel32 = GetModuleHandleA("kernel32.dll");
  if (!kernel32) {
    VirtualFreeEx(process_handle, remote_mem, 0, MEM_RELEASE);
    return false;
  }

  load_lib = GetProcAddress(kernel32, "LoadLibraryA");
  if (!load_lib) {
    VirtualFreeEx(process_handle, remote_mem, 0, MEM_RELEASE);
    return false;
  }

  thread = CreateRemoteThread(process_handle, NULL, 0,
                              (LPTHREAD_START_ROUTINE)(uintptr_t)load_lib,
                              remote_mem, 0, NULL);
  if (!thread) {
    VirtualFreeEx(process_handle, remote_mem, 0, MEM_RELEASE);
    return false;
  }

  WaitForSingleObject(thread, 5000);
  CloseHandle(thread);
  VirtualFreeEx(process_handle, remote_mem, 0, MEM_RELEASE);

  return true;
}

bool hook_eject_dll(HANDLE process_handle, const char *dll_name) {
  HMODULE modules[1024];
  DWORD cb_needed;
  int count, i;
  char module_path[MAX_PATH];

  if (!process_handle || !dll_name)
    return false;

  if (!EnumProcessModulesEx(process_handle, modules, sizeof(modules),
                            &cb_needed, 0)) {
    return false;
  }

  count = cb_needed / sizeof(HMODULE);

  for (i = 0; i < count; i++) {
    if (GetModuleFileNameExA(process_handle, modules[i], module_path,
                             MAX_PATH)) {
      char *fname = strrchr(module_path, '\\');
      if (fname) {
        fname++;
        if (_stricmp(fname, dll_name) == 0) {
          HMODULE kernel32 = GetModuleHandleA("kernel32.dll");
          FARPROC free_lib;
          HANDLE thread;

          if (!kernel32)
            return false;

          free_lib = GetProcAddress(kernel32, "FreeLibrary");
          if (!free_lib)
            return false;

          thread = CreateRemoteThread(
              process_handle, NULL, 0,
              (LPTHREAD_START_ROUTINE)(uintptr_t)free_lib, modules[i], 0, NULL);
          if (thread) {
            WaitForSingleObject(thread, 5000);
            CloseHandle(thread);
            return true;
          }
          return false;
        }
      }
    }
  }

  return false;
}

uint64_t hook_call_original(HookHandle *handle, const void *args,
                            size_t args_size) {
  (void)handle;
  (void)args;
  (void)args_size;
  return 0;
}

bool hook_write_jmp(void *target_address, void *hook_function,
                    uint8_t *original_bytes) {
  SIZE_T written = 0;
  uint8_t jmp_patch[14];
  uintptr_t addr;
  DWORD old_protect;

  if (!target_address || !hook_function || !original_bytes)
    return false;

  ReadProcessMemory(GetCurrentProcess(), target_address, original_bytes, 14,
                    &written);

  jmp_patch[0] = 0xFF;
  jmp_patch[1] = 0x25;
  jmp_patch[2] = 0x00;
  jmp_patch[3] = 0x00;
  jmp_patch[4] = 0x00;
  jmp_patch[5] = 0x00;

  addr = (uintptr_t)hook_function;
  memcpy(&jmp_patch[6], &addr, sizeof(uintptr_t));

  VirtualProtect(target_address, 14, PAGE_EXECUTE_READWRITE, &old_protect);
  WriteProcessMemory(GetCurrentProcess(), target_address, jmp_patch, 14,
                     &written);
  VirtualProtect(target_address, 14, old_protect, &old_protect);

  return true;
}

bool hook_restore_bytes(void *target_address, const uint8_t *original_bytes,
                        size_t size) {
  SIZE_T written = 0;
  DWORD old_protect;

  if (!target_address || !original_bytes || size == 0)
    return false;

  VirtualProtect(target_address, size, PAGE_EXECUTE_READWRITE, &old_protect);
  WriteProcessMemory(GetCurrentProcess(), target_address, original_bytes, size,
                     &written);
  VirtualProtect(target_address, size, old_protect, &old_protect);

  return true;
}

void *hook_get_module_base(HANDLE process_handle, const char *module_name) {
  return find_module_in_process(process_handle, module_name);
}