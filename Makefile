APP_NAME := dustrown
PREFIX ?= $(HOME)/.local
BINDIR ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share
ICONDIR ?= $(DATADIR)/icons/hicolor/scalable/apps
APPDIR ?= $(DATADIR)/applications
DESTDIR ?=
ICON_SOURCE ?= docs/icon.svg

.PHONY: build release install install-bin install-icon install-desktop uninstall run clean

build:
	cargo build

release:
	cargo build --release

install: release install-bin install-icon install-desktop

install-bin:
	install -d "$(DESTDIR)$(BINDIR)"
	install -m 0755 "target/release/$(APP_NAME)" "$(DESTDIR)$(BINDIR)/$(APP_NAME)"

install-icon:
	install -d "$(DESTDIR)$(ICONDIR)"
	install -m 0644 "$(ICON_SOURCE)" "$(DESTDIR)$(ICONDIR)/$(APP_NAME).svg"

install-desktop:
	install -d "$(DESTDIR)$(APPDIR)"
	printf '%s\n' \
		'[Desktop Entry]' \
		'Type=Application' \
		'Name=Dustrown' \
		'Comment=Dead simple Markdown viewer' \
		'Exec=$(BINDIR)/$(APP_NAME) %f' \
		'Icon=$(APP_NAME)' \
		'Terminal=false' \
		'Categories=Utility;Viewer;TextEditor;' \
		'MimeType=text/markdown;text/plain;' \
		> "$(DESTDIR)$(APPDIR)/$(APP_NAME).desktop"

uninstall:
	rm -f "$(DESTDIR)$(BINDIR)/$(APP_NAME)"
	rm -f "$(DESTDIR)$(ICONDIR)/$(APP_NAME).svg"
	rm -f "$(DESTDIR)$(APPDIR)/$(APP_NAME).desktop"

run:
	cargo run --

clean:
	cargo clean
