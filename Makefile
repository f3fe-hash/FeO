CMAKE   := cmake
MAKE    := make
OPENSSL := openssl

BUILD_DIR   := build
MKDIR       := mkdir
RMDIR       := rm

OUTPUT := feo_runtime
KEYS   := keys
KEY    := $(KEYS)/key.pem
CERT   := $(KEYS)/cert.pem

compile:
	@clear
	@$(RMDIR) -rf $(BUILD_DIR)
	@$(MKDIR) $(BUILD_DIR)
	@$(CMAKE) -B $(BUILD_DIR)
	@$(MAKE) -C $(BUILD_DIR)

run:
	@./$(BUILD_DIR)/$(OUTPUT)

client:
	@$(OPENSSL) s_client -connect 127.0.0.1:8080

gen_keys:
	@$(MKDIR) -p keys
	@$(OPENSSL) req -x509 -newkey rsa:4096 -keyout $(KEY) -out $(CERT) -sha256 -days 365 -nodes