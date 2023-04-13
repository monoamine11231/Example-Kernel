# This script is runned after the kernel was compiled into an ELF file
# No need for `parted`, since MBR is static and hold a primary partition for 512*100000 byte img file
#!/bin/sh
KERNEL_FILE_ELF=target/x86_64-peepo/debug/kernel
KERNEL_FILE_BIN=build/kernel.bin
OS=`uname -s`

command_exists() {
    command -v "$1" > /dev/null 2>&1
}

if ! command_exists objcopy; then
    echo 'INSTALL OBJCOPY IN BINUTILS (MacOS)'
    exit 1
fi

if ! command_exists mkfs.fat; then
    echo 'INSTALL MKFS.FAT IN DOSFS (MacOS)'
    exit 1
fi


objcopy -I elf64-x86-64 -O binary --binary-architecture=i386:x86-64 $KERNEL_FILE_ELF $KERNEL_FILE_BIN

if [ $OS = "Linux" ]
then
    KERNEL_SIZE=$(stat -c%s "$KERNEL_FILE_BIN")
elif [ $OS = "Darwin" ]
then
    KERNEL_SIZE=$(stat -f%z "$KERNEL_FILE_BIN")
elif [ $OS = "FreeBSD" ]
then
    echo "FreeBSD is currently not supported"
    exit 1
else
    echo "CRITICAL: Unknown operating system"
    exit 1
fi

# The number of sectors (512 bytes) that the kernel size has
KERNEL_SECTORS=$(($((KERNEL_SIZE+511))/512))

dd if=/dev/zero of=build/os.img bs=512 count=100000
# +2 sectors for VBR and FSInfo structs

mkfs.fat -b 0 -F 32 -M 0xf8 --mbr=n -R $((KERNEL_SECTORS+2)) --offset=1 build/os.img
dd if=build/bootloader/mbr.bin of=build/os.img bs=512 count=1 conv=notrunc
dd if=build/bootloader/vbr.bin of=build/os.img bs=1 count=420 conv=notrunc seek=602
dd if=build/kernel.bin of=build/os.img bs=512 count=$KERNEL_SECTORS seek=3 conv=notrunc
