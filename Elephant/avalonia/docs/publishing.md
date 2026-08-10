# Publication Avalonia Native AOT

Les profils de ce dossier préparent une publication native AOT, autonome et à fichier unique du projet `ElephantNote.Avalonia`. Ils sont volontairement séparés du `.csproj` existant.

Cette publication concerne uniquement les exécutables natifs Desktop/mobile.
Elle ne produit ni cible Avalonia Browser, ni bundle WebAssembly, ni serveur de
vault. La conception du futur chemin Web est décrite dans
[`architecture.md`](architecture.md) et reste explicitement non livrée.

## Profils et cibles

| Profil | RID | Sortie attendue |
| --- | --- | --- |
| `macos-arm64-aot.pubxml` | `osx-arm64` | `Elephant/avalonia/build/artifacts/publish/macos-arm64-aot/` |
| `windows-x64-aot.pubxml` | `win-x64` | `Elephant/avalonia/build/artifacts/publish/windows-x64-aot/` |
| `linux-x64-aot.pubxml` | `linux-x64` | `Elephant/avalonia/build/artifacts/publish/linux-x64-aot/` |

Chaque profil fixe `Release`, `PublishAot`, `SelfContained`, `PublishTrimmed`, `PublishSingleFile` et `StripSymbols`. Un RID est spécifique à un OS et une architecture; une publication pour Apple Silicon, par exemple, ne remplace pas une publication macOS Intel.

## Pré-requis

- .NET 8 SDK ou une version ultérieure compatible avec le projet;
- la toolchain native de la plateforme cible : Xcode Command Line Tools sur macOS, Visual Studio avec les outils C++ sur Windows, et clang ainsi que les paquets de développement requis par .NET sur Linux;
- un environnement propre avec les dépendances NuGet restaurées.

Native AOT analyse aussi le code et les dépendances au moment de la publication. Les avertissements de trimming, de chargement dynamique ou de XAML doivent être traités avant de considérer un binaire publiable. Cette arborescence ajoute les profils, mais ne constitue pas une preuve de publication ni de compatibilité runtime.

## Vérifier la frontière de dépendances

Depuis la racine du dépôt :

```bash
bash Elephant/avalonia/build/verify-no-web-dependencies.sh
```

Le garde-fou inspecte uniquement les fichiers qui portent des dépendances (`*.csproj`, `*.props`, `*.targets`, manifests et lockfiles) sous `Elephant/avalonia/`. Il échoue avec le code 1 dès qu'une référence Tauri, Vite ou Vue y est trouvée. Les textes explicatifs de `README.md` et de `docs/` ne sont pas scannés afin de ne pas confondre documentation de migration et dépendance réelle.

## Restaurer et publier

Restaurer la solution :

```bash
dotnet restore Elephant/avalonia/ElephantNote.Avalonia.sln
```

Publier macOS Apple Silicon depuis macOS :

```bash
dotnet publish Elephant/avalonia/src/ElephantNote.Avalonia/ElephantNote.Avalonia.csproj \
  -c Release -r osx-arm64 \
  -p:PublishProfileFullPath="$PWD/Elephant/avalonia/build/publish/macos-arm64-aot.pubxml"
```

Publier Windows x64 depuis Windows PowerShell :

```powershell
$profile = Join-Path (Get-Location) 'Elephant/avalonia/build/publish/windows-x64-aot.pubxml'
dotnet publish Elephant/avalonia/src/ElephantNote.Avalonia/ElephantNote.Avalonia.csproj `
  -c Release -r win-x64 `
  "-p:PublishProfileFullPath=$profile"
```

Publier Linux x64 depuis Linux :

```bash
dotnet publish Elephant/avalonia/src/ElephantNote.Avalonia/ElephantNote.Avalonia.csproj \
  -c Release -r linux-x64 \
  -p:PublishProfileFullPath="$PWD/Elephant/avalonia/build/publish/linux-x64-aot.pubxml"
```

Le `-r` répète volontairement le RID du profil : il rend la cible visible dans la commande et évite toute ambiguïté lors d'une invocation depuis un autre outil. Le chemin complet est utilisé car les `.pubxml` sont conservés sous `build/publish/`, et non sous `src/.../Properties/PublishProfiles/`.

## État de validation

Ces commandes sont documentées mais ne sont pas exécutées par cette modification. L'exécution réelle devra conserver la sortie de `dotnet publish`, les avertissements AOT/trimming et le contenu de chaque dossier `build/artifacts/publish/` avant toute revendication de binaire fonctionnel ou de compatibilité macOS, Windows ou Linux.

Références : [Native AOT Avalonia](https://docs.avaloniaui.net/docs/deployment/native-aot), [Native AOT .NET](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot) et [commande `dotnet publish`](https://learn.microsoft.com/en-us/dotnet/core/tools/dotnet-publish).
