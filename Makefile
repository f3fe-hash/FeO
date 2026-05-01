CMAKE := cmake
MAKE  := make

BUILD_DIR := build
MKDIR     := mkdir
RMDIR     := rm

OUTPUT := feo_runtime

compile:
	@clear
	@$(RMDIR) -r $(BUILD_DIR)
	@$(MKDIR) $(BUILD_DIR)
	@$(CMAKE) -B $(BUILD_DIR)
	@$(MAKE) -C $(BUILD_DIR)

run:
	@./$(BUILD_DIR)/$(OUTPUT)