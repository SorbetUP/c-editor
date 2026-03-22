// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'notes_provider.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

String _$notesHash() => r'738d2742f5b3bcf508472333cc3b607f507a9ee5';

/// See also [Notes].
@ProviderFor(Notes)
final notesProvider =
    AutoDisposeAsyncNotifierProvider<Notes, List<String>>.internal(
  Notes.new,
  name: r'notesProvider',
  debugGetCreateSourceHash:
      const bool.fromEnvironment('dart.vm.product') ? null : _$notesHash,
  dependencies: null,
  allTransitiveDependencies: null,
);

typedef _$Notes = AutoDisposeAsyncNotifier<List<String>>;
String _$noteDetailsHash() => r'af080e56cba322d71376b5f7e12f6c51e3b5fe44';

/// Copied from Dart SDK
class _SystemHash {
  _SystemHash._();

  static int combine(int hash, int value) {
    // ignore: parameter_assignments
    hash = 0x1fffffff & (hash + value);
    // ignore: parameter_assignments
    hash = 0x1fffffff & (hash + ((0x0007ffff & hash) << 10));
    return hash ^ (hash >> 6);
  }

  static int finish(int hash) {
    // ignore: parameter_assignments
    hash = 0x1fffffff & (hash + ((0x03ffffff & hash) << 3));
    // ignore: parameter_assignments
    hash = hash ^ (hash >> 11);
    return 0x1fffffff & (hash + ((0x00003fff & hash) << 15));
  }
}

abstract class _$NoteDetails
    extends BuildlessAutoDisposeAsyncNotifier<Document?> {
  late final String notePath;

  FutureOr<Document?> build(
    String notePath,
  );
}

/// See also [NoteDetails].
@ProviderFor(NoteDetails)
const noteDetailsProvider = NoteDetailsFamily();

/// See also [NoteDetails].
class NoteDetailsFamily extends Family<AsyncValue<Document?>> {
  /// See also [NoteDetails].
  const NoteDetailsFamily();

  /// See also [NoteDetails].
  NoteDetailsProvider call(
    String notePath,
  ) {
    return NoteDetailsProvider(
      notePath,
    );
  }

  @override
  NoteDetailsProvider getProviderOverride(
    covariant NoteDetailsProvider provider,
  ) {
    return call(
      provider.notePath,
    );
  }

  static const Iterable<ProviderOrFamily>? _dependencies = null;

  @override
  Iterable<ProviderOrFamily>? get dependencies => _dependencies;

  static const Iterable<ProviderOrFamily>? _allTransitiveDependencies = null;

  @override
  Iterable<ProviderOrFamily>? get allTransitiveDependencies =>
      _allTransitiveDependencies;

  @override
  String? get name => r'noteDetailsProvider';
}

/// See also [NoteDetails].
class NoteDetailsProvider
    extends AutoDisposeAsyncNotifierProviderImpl<NoteDetails, Document?> {
  /// See also [NoteDetails].
  NoteDetailsProvider(
    String notePath,
  ) : this._internal(
          () => NoteDetails()..notePath = notePath,
          from: noteDetailsProvider,
          name: r'noteDetailsProvider',
          debugGetCreateSourceHash:
              const bool.fromEnvironment('dart.vm.product')
                  ? null
                  : _$noteDetailsHash,
          dependencies: NoteDetailsFamily._dependencies,
          allTransitiveDependencies:
              NoteDetailsFamily._allTransitiveDependencies,
          notePath: notePath,
        );

  NoteDetailsProvider._internal(
    super._createNotifier, {
    required super.name,
    required super.dependencies,
    required super.allTransitiveDependencies,
    required super.debugGetCreateSourceHash,
    required super.from,
    required this.notePath,
  }) : super.internal();

  final String notePath;

  @override
  FutureOr<Document?> runNotifierBuild(
    covariant NoteDetails notifier,
  ) {
    return notifier.build(
      notePath,
    );
  }

  @override
  Override overrideWith(NoteDetails Function() create) {
    return ProviderOverride(
      origin: this,
      override: NoteDetailsProvider._internal(
        () => create()..notePath = notePath,
        from: from,
        name: null,
        dependencies: null,
        allTransitiveDependencies: null,
        debugGetCreateSourceHash: null,
        notePath: notePath,
      ),
    );
  }

  @override
  AutoDisposeAsyncNotifierProviderElement<NoteDetails, Document?>
      createElement() {
    return _NoteDetailsProviderElement(this);
  }

  @override
  bool operator ==(Object other) {
    return other is NoteDetailsProvider && other.notePath == notePath;
  }

  @override
  int get hashCode {
    var hash = _SystemHash.combine(0, runtimeType.hashCode);
    hash = _SystemHash.combine(hash, notePath.hashCode);

    return _SystemHash.finish(hash);
  }
}

mixin NoteDetailsRef on AutoDisposeAsyncNotifierProviderRef<Document?> {
  /// The parameter `notePath` of this provider.
  String get notePath;
}

class _NoteDetailsProviderElement
    extends AutoDisposeAsyncNotifierProviderElement<NoteDetails, Document?>
    with NoteDetailsRef {
  _NoteDetailsProviderElement(super.provider);

  @override
  String get notePath => (origin as NoteDetailsProvider).notePath;
}
// ignore_for_file: type=lint
// ignore_for_file: subtype_of_sealed_class, invalid_use_of_internal_member, invalid_use_of_visible_for_testing_member
