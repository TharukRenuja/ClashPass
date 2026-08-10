.PHONY: all dev build release run clean install uninstall

APP := clashpass

all: release

dev:
	cargo tauri dev

release:
	cargo tauri build

run: dev

clean:
	cargo clean
	rm -rf src-tauri/target

install: release
	install -Dm755 src-tauri/target/release/$(APP) $(DESTDIR)$(PREFIX)/bin/$(APP)
	install -Dm644 icons/clashpass_256.png $(DESTDIR)$(PREFIX)/share/icons/hicolor/256x256/apps/$(APP).png

uninstall:
	rm -f $(DESTDIR)$(PREFIX)/bin/$(APP)
	rm -f $(DESTDIR)$(PREFIX)/share/applications/$(APP).desktop
