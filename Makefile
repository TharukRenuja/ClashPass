.PHONY: all build release run clean install uninstall dist-tar dist-deb

APP := clashpass
BUILD := /tmp/$(APP)-build

all: release

release:
	mkdir -p $(BUILD)
	rsync -a --exclude target . $(BUILD)/
	$(MAKE) -C $(BUILD) _release

_release:
	cargo build --release
	cp target/release/$(APP) .

run: release
	./$(APP)

clean:
	rm -rf $(BUILD)
	cargo clean

install: release
	install -Dm755 $(APP) $(DESTDIR)$(PREFIX)/bin/$(APP)
	install -Dm644 icons/clashpass_256.png $(DESTDIR)$(PREFIX)/share/icons/hicolor/256x256/apps/$(APP).png
	install -Dm644 $(APP).desktop $(DESTDIR)$(PREFIX)/share/applications/$(APP).desktop

uninstall:
	rm -f $(DESTDIR)$(PREFIX)/bin/$(APP)
	rm -f $(DESTDIR)$(PREFIX)/share/applications/$(APP).desktop

dist-tar: release
	mkdir -p dist/$(APP)-v0.1.0
	cp $(BUILD)/target/release/$(APP) dist/$(APP)-v0.1.0/
	cp -r test_data dist/$(APP)-v0.1.0/
	cp icons/clashpass_256.png dist/$(APP)-v0.1.0/
	cd dist && tar czf ../dist/$(APP)-v0.1.0-linux-x86_64.tar.gz $(APP)-v0.1.0
	rm -rf dist/$(APP)-v0.1.0
	@echo "Created dist/$(APP)-v0.1.0-linux-x86_64.tar.gz"
