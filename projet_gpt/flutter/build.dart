#!/usr/bin/env dart
// Build script for multiplatform C Editor Flutter

import 'dart:io';
import 'dart:convert';

void main(List<String> args) async {
  final command = args.isNotEmpty ? args[0] : 'help';
  
  switch (command) {
    case 'native':
      await buildNative();
      break;
    case 'wasm':
      await buildWasm();
      break;
    case 'all':
      await buildNative();
      await buildWasm();
      break;
    case 'clean':
      await clean();
      break;
    default:
      printHelp();
  }
}

Future<void> buildNative() async {
  print('🔨 Building native libraries...');
  
  final platforms = <String, List<String>>{
    'ios': ['ios-arm64', 'ios-simulator-x64'],
    'android': ['android-arm64', 'android-x64', 'android-arm'],
    'macos': ['macos-arm64', 'macos-x64'],
    'windows': ['windows-x64'],
    'linux': ['linux-x64', 'linux-arm64'],
  };
  
  for (final entry in platforms.entries) {
    final platform = entry.key;
    final architectures = entry.value;
    
    print('  Building for $platform...');
    
    for (final arch in architectures) {
      try {
        await _buildForPlatform(platform, arch);
        print('    ✅ $arch');
      } catch (e) {
        print('    ❌ $arch: $e');
      }
    }
  }
  
  print('✅ Native libraries built');
}

Future<void> buildWasm() async {
  print('🌐 Building WASM module...');
  
  try {
    // Check if Emscripten is available
    final emccResult = await Process.run('emcc', ['--version'], runInShell: true);
    if (emccResult.exitCode != 0) {
      throw Exception('Emscripten not found. Please install Emscripten SDK.');
    }
    
    // Build WASM
    final buildArgs = [
      '../src/editor.c',
      '../src/markdown.c',
      '../src/json.c',
      '-o', 'web/editor.wasm',
      '-s', 'WASM=1',
      '-s', 'EXPORTED_FUNCTIONS=[\"_malloc\",\"_free\",\"_markdown_to_json\",\"_json_to_markdown\",\"_editor_init\",\"_editor_input\",\"_editor_get_document\",\"_editor_free\"]',
      '-s', 'EXPORTED_RUNTIME_METHODS=[\"ccall\",\"cwrap\"]',
      '-s', 'ALLOW_MEMORY_GROWTH=1',
      '-s', 'INITIAL_MEMORY=1MB',
      '-s', 'STACK_SIZE=512KB',
      '-O3',
      '-std=c11',
      '-Wall',
      '-Wextra',
      '-DWASM_BUILD=1',
    ];
    
    final result = await Process.run('emcc', buildArgs, runInShell: true);
    
    if (result.exitCode != 0) {
      throw Exception('WASM build failed: ${result.stderr}');
    }
    
    print('✅ WASM module built at web/editor.wasm');
    
  } catch (e) {
    print('❌ WASM build failed: $e');
    rethrow;
  }
}

Future<void> _buildForPlatform(String platform, String arch) async {
  final sourceFiles = [
    '../src/editor.c',
    '../src/markdown.c', 
    '../src/json.c',
  ];
  
  final commonFlags = [
    '-std=c11',
    '-Wall',
    '-Wextra',
    '-Werror',
    '-O3',
    '-fPIC',
    '-DFLUTTER_BUILD=1',
  ];
  
  switch (platform) {
    case 'ios':
      await _buildIOS(arch, sourceFiles, commonFlags);
      break;
    case 'android':
      await _buildAndroid(arch, sourceFiles, commonFlags);
      break;
    case 'macos':
      await _buildMacOS(arch, sourceFiles, commonFlags);
      break;
    case 'windows':
      await _buildWindows(arch, sourceFiles, commonFlags);
      break;
    case 'linux':
      await _buildLinux(arch, sourceFiles, commonFlags);
      break;
  }
}

Future<void> _buildIOS(String arch, List<String> sources, List<String> flags) async {
  final isSimulator = arch.contains('simulator');
  final sdk = isSimulator ? 'iphonesimulator' : 'iphoneos';
  final target = isSimulator ? 'x86_64-apple-ios-simulator' : 'arm64-apple-ios';
  
  final args = [
    '-arch', arch.contains('x64') ? 'x86_64' : 'arm64',
    '-isysroot', '\$(xcrun --sdk $sdk --show-sdk-path)',
    '-target', target,
    '-mios-version-min=11.0',
    ...flags,
    ...sources,
    '-o', 'ios/libeditor.a',
  ];
  
  final result = await Process.run('clang', args, runInShell: true);
  if (result.exitCode != 0) {
    throw Exception('iOS build failed: ${result.stderr}');
  }
}

Future<void> _buildAndroid(String arch, List<String> sources, List<String> flags) async {
  final ndkRoot = Platform.environment['ANDROID_NDK_ROOT'] ?? 
                  Platform.environment['NDK_ROOT'] ?? 
                  '/usr/local/android-ndk';
  
  if (!Directory(ndkRoot).existsSync()) {
    throw Exception('Android NDK not found. Set ANDROID_NDK_ROOT environment variable.');
  }
  
  final archMap = {
    'android-arm64': 'aarch64',
    'android-x64': 'x86_64', 
    'android-arm': 'arm',
  };
  
  final targetArch = archMap[arch]!;
  final toolchain = '$ndkRoot/toolchains/llvm/prebuilt/linux-x86_64/bin';
  final target = '$targetArch-linux-android21';
  
  final args = [
    '--target=$target',
    '-fPIC',
    '-shared',
    ...flags,
    ...sources,
    '-o', 'android/src/main/jniLibs/$targetArch/libeditor.so',
  ];
  
  final result = await Process.run('$toolchain/clang', args);
  if (result.exitCode != 0) {
    throw Exception('Android build failed: ${result.stderr}');
  }
}

Future<void> _buildMacOS(String arch, List<String> sources, List<String> flags) async {
  final targetArch = arch.contains('arm64') ? 'arm64' : 'x86_64';
  
  final args = [
    '-arch', targetArch,
    '-dynamiclib',
    '-install_name', '@rpath/libeditor.dylib',
    ...flags,
    ...sources,
    '-o', 'macos/libeditor.dylib',
  ];
  
  final result = await Process.run('clang', args);
  if (result.exitCode != 0) {
    throw Exception('macOS build failed: ${result.stderr}');
  }
}

Future<void> _buildWindows(String arch, List<String> sources, List<String> flags) async {
  // Requires clang on Windows or cross-compilation
  final args = [
    '-shared',
    '-o', 'windows/editor.dll',
    ...flags,
    ...sources,
  ];
  
  final result = await Process.run('clang', args, runInShell: true);
  if (result.exitCode != 0) {
    throw Exception('Windows build failed: ${result.stderr}');
  }
}

Future<void> _buildLinux(String arch, List<String> sources, List<String> flags) async {
  final args = [
    '-shared',
    '-fPIC',
    ...flags,
    ...sources,
    '-o', 'linux/libeditor.so',
  ];
  
  final result = await Process.run('gcc', args);
  if (result.exitCode != 0) {
    throw Exception('Linux build failed: ${result.stderr}');
  }
}

Future<void> clean() async {
  print('🧹 Cleaning build artifacts...');
  
  final filesToDelete = [
    'web/editor.wasm',
    'web/editor.js',
    'ios/libeditor.a',
    'macos/libeditor.dylib',
    'windows/editor.dll',
    'linux/libeditor.so',
  ];
  
  final dirsToClean = [
    'android/src/main/jniLibs',
  ];
  
  for (final file in filesToDelete) {
    try {
      await File(file).delete();
      print('  Deleted $file');
    } catch (e) {
      // File doesn't exist, ignore
    }
  }
  
  for (final dir in dirsToClean) {
    try {
      await Directory(dir).delete(recursive: true);
      print('  Cleaned $dir');
    } catch (e) {
      // Directory doesn't exist, ignore
    }
  }
  
  print('✅ Clean complete');
}

void printHelp() {
  print('C Editor Flutter Build System');
  print('');
  print('Usage: dart build.dart <command>');
  print('');
  print('Commands:');
  print('  native    Build native libraries for all platforms');
  print('  wasm      Build WASM module for web');
  print('  all       Build both native and WASM');
  print('  clean     Clean all build artifacts');
  print('  help      Show this help message');
  print('');
  print('Requirements:');
  print('  - clang/gcc for native builds');
  print('  - Emscripten SDK for WASM builds');
  print('  - Android NDK for Android builds');
  print('  - Xcode for iOS builds');
}