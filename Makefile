all: run

mbr.bin: mbr.s
	nasm mbr.s -f bin -o mbr.bin
vbr.bin: vbr.s
	nasm vbr.s -f bin -o vbr.bin
target/os/debug/os_fat32_impl: src/main.rs
	cargo build

kernel.bin: target/os/debug/os_fat32_impl
	cp target/os/debug/os_fat32_impl kernel.bin

os.img: mbr.bin vbr.bin kernel.bin
	dd if=/dev/zero of=os.img bs=512 count=100000
	parted os.img -s mklabel msdos mkpart primary fat32 1s 100%
	mkfs.fat -b 0 -F 32 -M 0xf8 --mbr=n -R 20 --offset=1 os.img
	dd if=mbr.bin of=os.img bs=440 count=1 conv=notrunc
	dd if=vbr.bin of=os.img bs=1 count=420 conv=notrunc seek=602
	dd if=test.bin of=os.img bs=512 count=18 seek=3 conv=notrunc

run: os.img
	qemu-system-x86_64 -drive format=raw,media=disk,file=os.img -monitor stdio -d int -no-reboot

clean:
	rm os.img mbr.bin vbr.bin kernel.bin