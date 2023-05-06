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
	sh makeimg_half.sh

run: os.img
	# sudo sh fstest.sh
	qemu-system-x86_64 -drive format=raw,media=disk,file=build/os.img -serial stdio -no-reboot -no-shutdown

debug: os.img
	qemu-system-x86_64 -drive format=raw,media=disk,file=build/os.img -serial stdio -d cpu_reset,guest_errors -no-reboot -no-shutdown -S -gdb tcp::9000

clean:
	rm build/kernel.bin build/os.img build/bootloader/mbr.bin build/bootloader/vbr.bin
