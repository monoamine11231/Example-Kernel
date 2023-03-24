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

![widePeepoHappy](peepo-emotes/widePeepoHappy.png "test image")
