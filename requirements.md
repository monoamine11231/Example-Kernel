# peepo64

Meeting: 
- Vid varje datorlabb/övning
- KTH/hos Movitz
- 

Timeplan:
- First two week: syscall contract and cargo setup
- following weeks: best effort
- May 19th MVP
- May 26th, integration etc
- -- polish


Ioannis: FS
Dima: userspace
Movitz: tasks/interupts/core
Daniel: memory/core
Anton: libPeepo / rust stdlib impl
Castor: FS
Adam: drivers
Seb: rust stdlib
??? -- tooling, debugger, stacktraces etc.

Tasks to distribute: 
- more core focus, userspace apps, own bootloader (if someone is interested)

# spec

Slutprodukt: kernel som köra något shell-liknande, förhoppningsvis mer.

- Rust implementation
- Long mode
- steal a bootloader

need to have
- FS capability
- some way of running a userspace application
- PS2 keyboard

nice to have
- syscall contract, enabling paralell kernel-/userspace development
- proper proccess management
- ELF loader
- ring 3 for procs

if we go mad
- networking (might exist stealable drivers)
- USB keyboard
- PIE

Memory
- OOM killer
- Page table
- Fault handler
- Page allocator (Buddy or Linked)
- KAlloc (slab allocator)
- 4 Level page directory entry tree PML, PML being the per-process page table. Kernel has one
- Data structure storing every PML
- Must colab with scheduler -- swapping out the page table register on context switch

PCI
- MMIO och/eller ports
- Pratar med SATA controller etc. 

Storage reading
- IDE eller AHCI

Scheduling
- APIC as timer
- Timingen av timern med en annan timer för att få timingen av timern
- Timer raises interrupts intermittently
- Round robin
- Context switching
- - Page table register, flags, program counter -- all registers
- - Context switch register

![widePeepoHappy](peepo-emotes/widePeepoHappy.png "test image")
