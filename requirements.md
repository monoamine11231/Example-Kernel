Meeting: 
- Vid varje datorlabb/övning
- KTH/hos Movitz
- 

Timeplan:
- First week: syscall contract
- following weeks: best effort


Ioannis: FS
Dima: userspace / libPeepo / rust stdlib impl
Movitz: tasks/interupts/core
Daniel: memory/core
Castor: ??
Adam: ??
Anton: ??
Seb: ??

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
- 

