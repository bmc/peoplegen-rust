BASE_DIR = $(HOME)

build:
	cargo build

release:
	cargo test
	cargo build -r

clean:
	cargo clean

install: release
	mkdir -p $(BASE_DIR)/bin
	cp target/release/peoplegen $(BASE_DIR)/bin
	mkdir -p $(BASE_DIR)/etc/peoplegen
	cp data/*.txt $(BASE_DIR)/etc/peoplegen
	@echo "Set these environment variables:"
	@echo "export PEOPLEGEN_MALE_FIRST_NAMES=\"$(BASE_DIR)/etc/peoplegen/male_first_names.txt\""
	@echo "export PEOPLEGEN_FEMALE_FIRST_NAMES=\"$(BASE_DIR)/etc/peoplegen/female_first_names.txt\""
	@echo "export PEOPLEGEN_LAST_NAMES=\"$(BASE_DIR)/etc/peoplegen/last_names.txt\""
