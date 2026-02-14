CC = gcc
CFLAGS = -std=c11 -Wall -Wextra -O2 -ffreestanding -fno-builtin -fno-stack-protector -nostdlib -I./include
AR = ar
ARFLAGS = rcs

INSTALL_DIR = ../lib
INCLUDE_DIR = $(INSTALL_DIR)/include
EXPORT_FILE = $(INSTALL_DIR)/summit.env

SRC_DIR = src
BUILD_DIR = build
OBJ_DIR = $(BUILD_DIR)/obj

IO_SRCS = $(wildcard $(SRC_DIR)/io/*.c)
STRING_SRCS = $(wildcard $(SRC_DIR)/string/*.c)
ALL_SRCS = $(IO_SRCS) $(STRING_SRCS)

IO_OBJS = $(patsubst $(SRC_DIR)/io/%.c,$(OBJ_DIR)/io/%.o,$(IO_SRCS))
STRING_OBJS = $(patsubst $(SRC_DIR)/string/%.c,$(OBJ_DIR)/string/%.o,$(STRING_SRCS))
ALL_OBJS = $(IO_OBJS) $(STRING_OBJS)

LIB = $(BUILD_DIR)/libstdlib.a

all: $(LIB)

$(LIB): $(ALL_OBJS) | $(BUILD_DIR)
	$(AR) $(ARFLAGS) $@ $^
	@echo "Built library: $@"

$(OBJ_DIR)/io/%.o: $(SRC_DIR)/io/%.c | $(OBJ_DIR)/io
	$(CC) $(CFLAGS) -c $< -o $@

$(OBJ_DIR)/string/%.o: $(SRC_DIR)/string/%.c | $(OBJ_DIR)/string
	$(CC) $(CFLAGS) -c $< -o $@

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

$(OBJ_DIR)/io:
	mkdir -p $(OBJ_DIR)/io

$(OBJ_DIR)/string:
	mkdir -p $(OBJ_DIR)/string

install: $(LIB)
	@mkdir -p $(INSTALL_DIR)
	@mkdir -p $(INCLUDE_DIR)
	@cp $(LIB) $(INSTALL_DIR)/
	@cp include/*.h $(INCLUDE_DIR)/
	@echo "# Summit standard library environment" > $(EXPORT_FILE)
	@echo "export SUMMIT_STDLIB_PATH=\"$(abspath $(INSTALL_DIR))\"" >> $(EXPORT_FILE)
	@echo "export LD_LIBRARY_PATH=\"$(abspath $(INSTALL_DIR)):\$$LD_LIBRARY_PATH\"" >> $(EXPORT_FILE)
	@echo "To set up environment, run: source $(EXPORT_FILE)"

install-global: install
	@if [ -f ~/.zshrc ]; then \
		echo "" >> ~/.zshrc; \
		echo "# Summit standard library" >> ~/.zshrc; \
		echo "export SUMMIT_STDLIB_PATH=\"$(abspath $(INSTALL_DIR))\"" >> ~/.zshrc; \
		echo "export LD_LIBRARY_PATH=\"$(abspath $(INSTALL_DIR)):\$$LD_LIBRARY_PATH\"" >> ~/.zshrc; \
	else \
		echo "" >> ~/.bashrc; \
		echo "# Summit standard library" >> ~/.bashrc; \
		echo "export SUMMIT_STDLIB_PATH=\"$(abspath $(INSTALL_DIR))\"" >> ~/.bashrc; \
		echo "export LD_LIBRARY_PATH=\"$(abspath $(INSTALL_DIR)):\$$LD_LIBRARY_PATH\"" >> ~/.bashrc; \
	fi
	@echo "" >> ~/.profile
	@echo "# Summit standard library" >> ~/.profile
	@echo "export SUMMIT_STDLIB_PATH=\"$(abspath $(INSTALL_DIR))\"" >> ~/.profile
	@if [ -f ~/.zshrc ]; then \
		echo "Environment added. Reload with: source ~/.zshrc"; \
	else \
		echo "Environment added. Reload with: source ~/.bashrc"; \
	fi

uninstall-global:
	@rm -rf $(INSTALL_DIR)
	@if [ -f ~/.zshrc ]; then \
		sed -i '/# Summit standard library/,+2d' ~/.zshrc; \
	fi
	@if [ -f ~/.bashrc ]; then \
		sed -i '/# Summit standard library/,+2d' ~/.bashrc; \
	fi
	@if [ -f ~/.profile ]; then \
		sed -i '/# Summit standard library/,+1d' ~/.profile; \
	fi
	@echo "Standard library uninstalled. Restart your shell."

clean:
	rm -rf $(BUILD_DIR)

rebuild: clean all

.PHONY: all clean rebuild install install-global uninstall-global