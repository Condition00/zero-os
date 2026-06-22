# Changelog

## 2026-01-24

### Refactoring

* Refactored the kernel source tree and reorganized project directories.
* Cleaned up code structure to improve maintainability.

### Current State

* Basic kernel is functional.
* Implemented a simple RAM-based filesystem (RamFS).
* Added a basic shell interface.

---

## 2026-01-25

### Userspace Foundation

* Began implementing userspace memory management and syscall infrastructure.
* Investigated kernel memory remapping approaches but encountered persistent page faults.
* Identified the issue and switched to allocating a dedicated userspace virtual address space.

### Privilege Level Transition

* Successfully implemented Ring 0 → Ring 3 transitions.
* Created the required `iretq` stack frame for entering userspace execution.
* Established the foundation for running user programs.

---

## 2026-01-26

### System Calls

* Completed syscall support between Ring 3 and Ring 0.
* Implemented a syscall handler with initial system call functionality.
* Added assembly glue code to match the System V AMD64 calling convention.
* User programs can now invoke kernel services through the syscall interface.

### Notes

* System Call Extensions (SCE) support still needs to be enabled/configured.
* Core userspace execution infrastructure is now operational.

---

## 2026-01-27

### libc Integration

* Started work on OS-specific userspace dependencies using a forked `mlibc` submodule.
* Attempted full `mlibc` integration but encountered significant build complexity.
* Installed only the required header files as an intermediate step.

### Future Work

* Develop a minimal libc implementation using the imported headers.
* Continue building the userspace runtime environment.

---

## 2026-02-14

### Assembly & Runtime Work

* Spent time learning x86_64 assembly to better understand low-level userspace/kernel interactions.
* Began exploring custom syscall wrapper implementations.
* Removed the incomplete `mlibc` integration from the project.

### Hardware Support

* Added PS/2 mouse support to the kernel.
* test


## Process refactor

* Process now owns a saved Context instead of separate entry/stack fields, and there is a global PROCESS_TABLE ready for later scheduling work. The userspace runner was updated to read rip and rsp from the new context shape.

* context.rs: holds the saved register state.
* process.rs: now stores pid, context, and state.
* mod.rs: exposes PROCESS_TABLE and add_process(...).
* runner.rs: now launches from process.context.