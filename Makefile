VERSION := $(shell cat VERSION | tr -d '[:space:]')

.PHONY: version-update
version-update:
	@echo "Syncing version $(VERSION) to all files..."
	@sed -i '' 's/^version = ".*"/version = "$(VERSION)"/' Cargo.toml
	@sed -i '' 's/"version": ".*"/"version": "$(VERSION)"/' crates/plugmux-app/src-tauri/tauri.conf.json
	@sed -i '' 's/"version": ".*"/"version": "$(VERSION)"/' crates/plugmux-app/package.json
	@echo "  VERSION:         $(VERSION)"
	@echo "  Cargo.toml:      $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')"
	@echo "  tauri.conf.json: $(shell grep '"version"' crates/plugmux-app/src-tauri/tauri.conf.json | head -1 | sed 's/.*"\([^"]*\)".*/\1/')"
	@echo "  package.json:    $(shell grep '"version"' crates/plugmux-app/package.json | head -1 | sed 's/.*"\([^"]*\)".*/\1/')"
	@echo ""
	@git add VERSION Cargo.toml crates/plugmux-app/src-tauri/tauri.conf.json crates/plugmux-app/package.json
	@git commit -m "chore: bump version to $(VERSION)"
	@echo "Done. Committed version $(VERSION)"
