# woflOS Changelog

## v0.4.0 - Layer 1: Privilege Transitions (2025-10-15) ✅
**MILESTONE: First user mode execution!**

**Achievements:**
- ✅ Process structure with PID, context, state tracking
- ✅ CPU context switching (31 registers + PC + sstatus)
- ✅ Syscall interface with 4 syscalls:
  - SYS_PUTC (1): Write character to console
  - SYS_EXIT (2): Exit process
  - SYS_GETPID (3): Get process ID
  - SYS_YIELD (4): Yield CPU to scheduler
- ✅ User mode transitions (S-mode ↔ U-mode)
- ✅ First userspace program (init process)
- ✅ Trap handler syscall dispatcher
- ✅ Context save/restore on every trap
- ✅ PC advancement after syscalls

**New Components:**
- `src/process/mod.rs`: Process management
- `src/process/context.rs`: CPU context structure
- `src/syscall/mod.rs`: Syscall interface
- `src/user/mod.rs`: User module wrapper
- `src/user/init.rs`: First userspace program
- `LAYER1_DEPLOYMENT.md`: Comprehensive guide
- `SYSCALL_REFERENCE.md`: Syscall documentation

**Technical Details:**
- Context structure: 31 GP registers + 2 special (PC, sstatus)
- User memory: 0x87000000-0x87010000 (64KB)
- Syscall detection: scause == 8 (U-mode ecall)
- Privilege: sstatus.SPP bit controls S/U mode

**Bug Fixes:**
- Fixed context conversion between stack and structure
- Added PC advancement after syscall (4 bytes)
- Cleared all registers on U-mode entry (security)

---

## v0.3.0 - Layer 0: Trap Handling (2025-10-15) ✅
**MILESTONE: Interrupts working!**

**Achievements:**
- ✅ Timer interrupts working (1Hz, stable)
- ✅ Full trap handler with context switching
- ✅ All 31 registers saved/restored on interrupt
- ✅ Exception dispatcher (interrupt vs exception)
- ✅ Hex number printing in interrupt context
- ✅ SBI timer calls working correctly

**Bug Fixes:**
- Fixed trap handler register save/restore
- Fixed stack alignment issues
- Fixed SBI ecall clobber lists
- Resolved file sync issues between VS Code and WSL

---

## v0.2.0 - Memory Management (2025-10-13) ✅
**MILESTONE: Dynamic allocation!**

**Achievements:**
- ✅ Frame allocator (bitmap-based, 4KB pages)
- ✅ Kernel heap allocator (bump allocator, 64KB)
- ✅ Memory initialization on boot
- ✅ BSS section clearing
- ✅ Rust `alloc` crate support (Vec, Box, etc.)

**Technical Details:**
- Physical memory: Bitmap allocator for 4KB frames
- Heap: 64KB bump allocator (no deallocation)
- Atomic operations for thread-safety (future SMP)
- Memory layout: 0x80200000-0x88000000 (128MB)

---

## v0.1.0 - First Boot (2025-10-12) ✅
**MILESTONE: Bare metal boot!**

**Achievements:**
- ✅ Bare metal boot on RISC-V
- ✅ UART driver (16550-compatible)
- ✅ Serial console output
- ✅ Basic panic handler
- ✅ Power-efficient idle loop (wfi)
- ✅ Linker script and memory layout
- ✅ QEMU virt machine support

**Technical Details:**
- Entry point: `_start` in `.text.boot` section
- UART: 0x10000000 (QEMU virt machine)
- Memory: 128MB at 0x80000000
- Kernel load: 0x80200000 (after OpenSBI)

---

## Roadmap

### ✅ Layer 0: Trap Handling (COMPLETE)
- Boot sequence ✓
- Memory/heap allocation ✓
- Timer interrupts ✓
- Exception handling ✓
- Dispatcher setup ✓

### ✅ Layer 1: Privilege Transitions (COMPLETE)
- Context switching ✓
- Supervisor/user modes ✓
- Syscall interface ✓
- First user program ✓

### 🚧 Layer 2: Process Isolation (NEXT)
- [ ] PMP configuration
- [ ] User memory isolation
- [ ] Multiple processes
- [ ] Process lifecycle management

### 📋 Layer 3: Scheduling (PLANNED)
- [ ] Round-robin scheduler
- [ ] Timer-based preemption
- [ ] Process priority
- [ ] Context switch optimization

### 📋 Layer 4: IPC Foundation (PLANNED)
- [ ] Synchronous message passing
- [ ] Kernel message buffers
- [ ] Endpoint abstraction
- [ ] Send/receive syscalls

### 📋 Layer 5: Capabilities (PLANNED)
- [ ] Capability structure
- [ ] Ed25519 crypto signing
- [ ] Syscall verification
- [ ] Capability passing via IPC
- [ ] Memory-as-capabilities model

---

## Statistics

**Total Development Time:** ~3 days  
**Total Code:** ~2500 lines  
**Languages:** Rust (95%), Assembly (5%)  
**Architecture:** RISC-V 64-bit  
**Target:** QEMU virt machine  

**Lines by Module:**
- Memory: ~400 lines
- Interrupts: ~200 lines
- Process: ~400 lines
- Syscall: ~150 lines
- User: ~300 lines
- Main/Boot: ~150 lines
- Drivers (UART): ~100 lines

---

**Architecture:** RISC-V 64-bit  
**Kernel Type:** Microkernel  
**Language:** Rust + Assembly  
**Platform:** QEMU virt machine (128MB RAM)  
**Security Model:** Capability-based (in progress)

**Built with 🐺 by wofl**  
*"One layer at a time, we build the future!"*