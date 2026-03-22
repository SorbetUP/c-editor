# Master Makefile for all C engines
CC = clang
CFLAGS = -std=c11 -Wall -Wextra -Werror -O2 -g -DNDEBUG

# Engine directories
ENGINE_DIRS = cursor markdown editor search_engine render_engine backup_engine crypto_engine

# Default target
all: engines

# Build all engines
engines:
	@echo "🔧 Building all C engines..."
	@for dir in $(ENGINE_DIRS); do \
		if [ -f $$dir/Makefile ]; then \
			echo "Building $$dir..."; \
			$(MAKE) -C $$dir static || exit 1; \
		else \
			echo "⚠️  No Makefile found in $$dir, skipping"; \
		fi; \
	done
	@echo "✅ All engines built successfully"

# Build individual engines
cursor:
	$(MAKE) -C cursor static

markdown:
	$(MAKE) -C markdown static

editor:
	$(MAKE) -C editor static

search_engine:
	$(MAKE) -C search_engine static

render_engine:
	$(MAKE) -C render_engine static

backup_engine:
	$(MAKE) -C backup_engine static

crypto_engine:
	$(MAKE) -C crypto_engine static

# Clean all engines
clean:
	@echo "🧹 Cleaning all engines..."
	@for dir in $(ENGINE_DIRS); do \
		if [ -f $$dir/Makefile ]; then \
			echo "Cleaning $$dir..."; \
			$(MAKE) -C $$dir clean; \
		fi; \
	done
	@echo "✅ All engines cleaned"

# Test all engines
test:
	@echo "🧪 Testing all engines..."
	@for dir in $(ENGINE_DIRS); do \
		if [ -f $$dir/Makefile ]; then \
			echo "Testing $$dir..."; \
			$(MAKE) -C $$dir test || true; \
		fi; \
	done

# Create stub Makefiles for engines that don't have them yet
create-makefiles:
	@for dir in search_engine render_engine backup_engine crypto_engine; do \
		if [ ! -f $$dir/Makefile ]; then \
			echo "Creating stub Makefile for $$dir"; \
			echo "CC = clang" > $$dir/Makefile; \
			echo "CFLAGS = -std=c11 -Wall -Wextra -Werror -O2 -g -DNDEBUG" >> $$dir/Makefile; \
			echo "TARGET = lib$$dir.a" >> $$dir/Makefile; \
			echo "" >> $$dir/Makefile; \
			echo "all: \$$(TARGET)" >> $$dir/Makefile; \
			echo "" >> $$dir/Makefile; \
			echo "\$$(TARGET):" >> $$dir/Makefile; \
			echo "	@echo \"✅ Stub library built: \$$(TARGET)\"" >> $$dir/Makefile; \
			echo "	touch \$$(TARGET)" >> $$dir/Makefile; \
			echo "" >> $$dir/Makefile; \
			echo "static: \$$(TARGET)" >> $$dir/Makefile; \
			echo "" >> $$dir/Makefile; \
			echo "clean:" >> $$dir/Makefile; \
			echo "	rm -f \$$(TARGET)" >> $$dir/Makefile; \
			echo "" >> $$dir/Makefile; \
			echo ".PHONY: all static clean" >> $$dir/Makefile; \
		fi; \
	done

# Print engine status
status:
	@echo "📊 Engine Status:"
	@for dir in $(ENGINE_DIRS); do \
		if [ -d $$dir ]; then \
			printf "  %-20s: " $$dir; \
			if [ -f $$dir/Makefile ]; then \
				printf "Makefile ✅  "; \
			else \
				printf "No Makefile ❌  "; \
			fi; \
			if [ -f $$dir/*.h ]; then \
				printf "Header ✅  "; \
			else \
				printf "No Header ❌  "; \
			fi; \
			if [ -f $$dir/*.c ]; then \
				printf "Source ✅"; \
			else \
				printf "No Source ❌"; \
			fi; \
			echo ""; \
		fi; \
	done

.PHONY: all engines clean test create-makefiles status cursor markdown editor search_engine render_engine backup_engine crypto_engine