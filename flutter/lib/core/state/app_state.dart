import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../storage/storage_service.dart';
import 'package:c_editor_flutter/models/models.dart';

part 'app_state.g.dart';

@riverpod
class AppState extends _$AppState {
  @override
  FutureOr<AppConfig> build() async {
    return _initialize();
  }

  Future<AppConfig> _initialize() async {
    try {
      // Initialize storage service
      final storageService = ref.read(storageServiceProvider);
      await storageService.initialize();

      // Load app configuration
      final config = await storageService.loadConfig();
      
      return config;
    } catch (e) {
      throw Exception('Failed to initialize app: $e');
    }
  }

  Future<void> initialize() async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(() => _initialize());
  }

  Future<void> updateConfig(AppConfig config) async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(() async {
      final storageService = ref.read(storageServiceProvider);
      await storageService.saveConfig(config);
      return config;
    });
  }
}

// Current note state
@riverpod
class CurrentNoteState extends _$CurrentNoteState {
  @override
  String? build() => null;

  void setCurrentNote(String? notePath) {
    state = notePath;
  }

  void clearCurrentNote() {
    state = null;
  }
}

// UI state
@riverpod
class UIState extends _$UIState {
  @override
  UIData build() => const UIData();

  void setLoading(bool loading) {
    state = state.copyWith(isLoading: loading);
  }

  void setError(String? error) {
    state = state.copyWith(error: error);
  }

  void clearError() {
    state = state.copyWith(error: null);
  }

  void showSnackBar(String message) {
    state = state.copyWith(snackBarMessage: message);
  }

  void clearSnackBar() {
    state = state.copyWith(snackBarMessage: null);
  }
}

class UIData {
  final bool isLoading;
  final String? error;
  final String? snackBarMessage;

  const UIData({
    this.isLoading = false,
    this.error,
    this.snackBarMessage,
  });

  UIData copyWith({
    bool? isLoading,
    String? error,
    String? snackBarMessage,
  }) {
    return UIData(
      isLoading: isLoading ?? this.isLoading,
      error: error ?? this.error,
      snackBarMessage: snackBarMessage ?? this.snackBarMessage,
    );
  }
}