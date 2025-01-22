SHELL := /bin/bash

# Default install paths:
LOCAL_BIN  = $(HOME)/.local/bin
GLOBAL_BIN = /usr/local/bin

# Detect the user's default shell profile for PATH config
# Adjust or add more checks (e.g. .zshrc, .profile, etc.) as you prefer
ifeq ($(shell basename $(SHELL)), zsh)
  USER_SHELL_PROFILE = $(HOME)/.zshrc
else
  USER_SHELL_PROFILE = $(HOME)/.bashrc
endif

.PHONY: all build install

all: build

build:
	cargo build --release

install: build
	@echo "Installing rrdeep..."
	@if [ `id -u` = 0 ]; then \
	  echo "Running as root; installing to $(GLOBAL_BIN)"; \
	  cp target/release/rrdeep $(GLOBAL_BIN)/rrdeep; \
	  chmod +x $(GLOBAL_BIN)/rrdeep; \
	  echo "rrdeep installed to $(GLOBAL_BIN)"; \
	else \
	  echo "Not running as root; installing to $(LOCAL_BIN)"; \
	  mkdir -p $(LOCAL_BIN); \
	  cp target/release/rrdeep $(LOCAL_BIN)/rrdeep; \
	  chmod +x $(LOCAL_BIN)/rrdeep; \
	  echo "rrdeep installed to $(LOCAL_BIN)"; \
	  if ! grep -q '$(LOCAL_BIN)' $(USER_SHELL_PROFILE) 2>/dev/null; then \
	    echo "" >> $(USER_SHELL_PROFILE); \
	    echo "# Added by rrdeep Makefile" >> $(USER_SHELL_PROFILE); \
	    echo "export PATH=\"$(LOCAL_BIN):\$$PATH\"" >> $(USER_SHELL_PROFILE); \
	    echo "Appended 'export PATH=$(LOCAL_BIN):\$PATH' to $(USER_SHELL_PROFILE)"; \
	    echo "Restart your shell or run: source $(USER_SHELL_PROFILE)"; \
	  else \
	    echo "$(LOCAL_BIN) is already in PATH in $(USER_SHELL_PROFILE)"; \
	  fi; \
	fi
