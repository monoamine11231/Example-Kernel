build() {
	sudo docker build -t kernel .
	CONTAINER=$(sudo docker create kernel)
	sudo docker cp ${CONTAINER}:/kernel_make/build ./build
}
run() {
	qemu-system-x86_64 -drive format=raw,media=disk,file=build/os.img -serial stdio -no-reboot -no-shutdown
}
clean() {
	# may mess with other containers
	# docker rm -f $(docker ps -a -q)
	echo iou
}

eval $1
