# This script is runned after the kernel was compiled into an ELF file 
KERNEL_FILE=target/x86_64-peepo/debug/kernel
KERNEL_SIZE=$(stat -c%s "$KERNEL_FILE")

# The number of sectors (512 bytes) that the kernel size has
KERNEL_SECTORS=$(($((KERNEL_SIZE+511))/512))

dd if=/dev/zero of=os.img bs=512 count=100000
parted os.img -s mklabel msdos mkpart primary fat32 1s 100%
# +2 sectors for VBR and FSInfo structs
mkfs.fat -b 0 -F 32 -M 0xf8 --mbr=n -R $((KERNEL_SECTORS+2)) --offset=1 os.img
dd if=mbr.bin of=os.img bs=440 count=1 conv=notrunc
dd if=vbr.bin of=os.img bs=1 count=420 conv=notrunc seek=602
objcopy -I elf64-x86-64 -O binary --binary-architecture=i386:x86-64 $KERNEL_FILE kernel.bin
dd if=kernel.bin of=os.img bs=512 count=$KERNEL_SECTORS seek=3 conv=notrunc