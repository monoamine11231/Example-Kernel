sudo mount -o loop,offset=512 build/os.img /mnt/tmp
sudo mkdir /mnt/tmp/kek/
sudo mkdir /mnt/tmp/kek/aba/
sudo echo "test1" > /mnt/tmp/lol.txt
sudo echo "test2" > /mnt/tmp/kek/lol2.txt
sudo echo "hello from FAT32" > /mnt/tmp/kek/aba/lol3.txt

sudo umount /mnt/tmp