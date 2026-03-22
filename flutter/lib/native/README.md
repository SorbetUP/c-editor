# Native Libraries

This directory contains the compiled native libraries for different platforms:

- `macos/libeditor.dylib` - macOS dynamic library
- `linux/libeditor.so` - Linux shared object 
- `windows/editor.dll` - Windows dynamic link library

## Building

To build the native libraries from the C core:

```bash
# From the root c-editor directory
cd src
make clean
make

# Copy to Flutter project
cp libeditor.a ../flutter/lib/native/macos/libeditor.dylib  # (convert to dylib)
# etc. for other platforms
```

## TODO

- Set up automated builds for all target platforms
- Add CMake configuration for cross-compilation
- Integrate with CI/CD pipeline for automatic library updates