// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'app_state.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

String _$appStateHash() => r'a428858bee75f0043c04bca71fdf0b493dc185bc';

/// See also [AppState].
@ProviderFor(AppState)
final appStateProvider =
    AutoDisposeAsyncNotifierProvider<AppState, AppConfig>.internal(
  AppState.new,
  name: r'appStateProvider',
  debugGetCreateSourceHash:
      const bool.fromEnvironment('dart.vm.product') ? null : _$appStateHash,
  dependencies: null,
  allTransitiveDependencies: null,
);

typedef _$AppState = AutoDisposeAsyncNotifier<AppConfig>;
String _$currentNoteStateHash() => r'30c6f2a035edfed8847f7cfe41cb48b1895dfd67';

/// See also [CurrentNoteState].
@ProviderFor(CurrentNoteState)
final currentNoteStateProvider =
    AutoDisposeNotifierProvider<CurrentNoteState, String?>.internal(
  CurrentNoteState.new,
  name: r'currentNoteStateProvider',
  debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
      ? null
      : _$currentNoteStateHash,
  dependencies: null,
  allTransitiveDependencies: null,
);

typedef _$CurrentNoteState = AutoDisposeNotifier<String?>;
String _$uIStateHash() => r'7481a076194fb62c54f39c97c607d43c04f15f1b';

/// See also [UIState].
@ProviderFor(UIState)
final uIStateProvider = AutoDisposeNotifierProvider<UIState, UIData>.internal(
  UIState.new,
  name: r'uIStateProvider',
  debugGetCreateSourceHash:
      const bool.fromEnvironment('dart.vm.product') ? null : _$uIStateHash,
  dependencies: null,
  allTransitiveDependencies: null,
);

typedef _$UIState = AutoDisposeNotifier<UIData>;
// ignore_for_file: type=lint
// ignore_for_file: subtype_of_sealed_class, invalid_use_of_internal_member, invalid_use_of_visible_for_testing_member
