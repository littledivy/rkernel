TARGET=.cargo/x86_64-kernel.json
BOOT_SOURCE=(multiboot_header long_mode_start boot)
set -e
cargo build --target $TARGET

BOOT_OUT=()

for entry in ${BOOT_SOURCE[@]};
do
  nasm -felf64 boot/$entry.asm
  BOOT_OUT+=(boot/$entry.o)
done


ld -n --gc-sections -T linker.ld -o kernel.bin boot/boot.o boot/multiboot_header.o boot/long_mode_start.o target/x86_64-kernel/debug/librust_kernel.a 
# rm -rf build/
# mkdir -p build/boot/grub
mv kernel.bin build/boot/kernel.bin
# cp boot/grub.cfg boot/boot/grub/grub.cfg

# dd if=/dev/zero of=harddisk.img bs=1M count=35
# mformat -F -i harddisk.img

grub-mkrescue -o x86_64-rkernel.iso build/
qemu-system-x86_64 -drive file=harddisk.img,format=raw,media=disk -cdrom x86_64-rkernel.iso -boot d
