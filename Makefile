all: run

mbr.bin: bootloader/mbr.s
	mkdir -p build/bootloader
	nasm bootloader/mbr.s -f bin -o build/bootloader/mbr.bin
vbr.bin: bootloader/vbr.s
	mkdir -p build/bootloader
	nasm bootloader/vbr.s -f bin -o build/bootloader/vbr.bin

cargo:
	cargo build

os.img: cargo mbr.bin vbr.bin
	sh makeimg.sh

run: os.img
	qemu-system-x86_64 -drive format=raw,media=disk,file=build/os.img -monitor stdio -d cpu_reset -no-reboot

clean:
	rm build/kernel.bin build/os.img build/bootloader/mbr.bin build/bootloader/vbr.bin