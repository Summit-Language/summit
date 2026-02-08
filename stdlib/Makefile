CC = gcc
CFLAGS = -fPIC -O3 -Wall -Wextra -ffreestanding -fno-builtin -fno-stack-protector
LDFLAGS = -shared -nostdlib

LIB_NAME = libsummit_std.so
SOURCES = io.c
OBJECTS = $(SOURCES:.c=.o)
HEADER = freestanding.h

INSTALL_DIR = ../lib
INCLUDE_DIR = $(INSTALL_DIR)/include
EXPORT_FILE = $(INSTALL_DIR)/summit.env

all: $(LIB_NAME)

$(LIB_NAME): $(OBJECTS)
	$(CC) $(LDFLAGS) -o $@ $^

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f $(OBJECTS) $(LIB_NAME) $(EXPORT_FILE)

install: $(LIB_NAME)
	@mkdir -p $(INSTALL_DIR)
	@mkdir -p $(INCLUDE_DIR)
	@cp $(LIB_NAME) $(INSTALL_DIR)/
	@cp $(HEADER) $(INCLUDE_DIR)/
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

.PHONY: all clean install install-global uninstall-global