CC=riscv64-unknown-elf-gcc

main:
	mkdir -p bin
	$(CC) -o bin/main_contract src/main_contract.c
	cargo run -- bin/main_contract | tee
