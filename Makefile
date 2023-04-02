all: run

mbr.bin: mbr.s
	nasm mbr.s -f bin -o mbr.bin
vbr.bin: vbr.s
	nasm vbr.s -f bin -o vbr.bin

cargo:
	cargo build

os.img: cargo mbr.bin vbr.bin
	
	sh makeimg.sh

run: os.img
	qemu-system-x86_64 -drive format=raw,media=disk,file=os.img -monitor stdio -d cpu_reset -no-reboot

clean:
	rm os.img mbr.bin vbr.bin kernel.bin
	cargo clean