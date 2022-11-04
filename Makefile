
build:
	cargo build

release:
	cargo test
	cargo build -r

clean:
	cargo clean

install: release
	cp target/release/peoplegen ~/bin
	mkdir -p ~/etc/peoplegen
	cp data/*.txt ~/etc/peoplegen
	@echo "Set these environment variables:"
	@echo "export PEOPLEGEN_MALE_FIRST_NAMES=\"$(HOME)/etc/peoplegen/male_first_names.txt\""
	@echo "export PEOPLEGEN_FEMALE_FIRST_NAMES=\"$(HOME)/etc/peoplegen/female_first_names.txt\""
	@echo "export PEOPLEGEN_LAST_NAMES=\"$(HOME)/etc/peoplegen/last_names.txt\""
