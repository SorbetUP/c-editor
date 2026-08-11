# Freya/Tauri PNG differential comparator

`compare.mjs` compare deux captures PNG produites par le même parcours utilisateur. Il compare les checkpoints statiques et chaque frame temporelle séparément, vérifie les dimensions, mesure les pixels décodés et écrit un diff PNG pour chaque image différente. `sharp` est utilisé quand il est déjà résolvable ; sinon le comparateur utilise un décodeur/encodeur PNG 8-bit intégré et signale les encodages PNG qu’il ne sait pas traiter.

## Organisation des captures

Les deux répertoires doivent avoir exactement les mêmes chemins PNG :

```text
captures/
  startup/static.png
  editor-open/frames/frame-000.png
  editor-open/frames/frame-001.png
```

Un fichier sous `frames/`, `temporal/` ou `motion/`, ou dont le nom est `frame-000.png`/`snapshot-000.png`, est classé `temporal`. Les autres PNG sont `static`. Un fichier manquant ou supplémentaire fait échouer la comparaison.

## Utilisation

```bash
node tools/freya-differential/compare.mjs \
  artifacts/tauri \
  artifacts/freya \
  --reference-label tauri \
  --candidate-label freya \
  --threshold 0 \
  --max-different-ratio 0 \
  --report artifacts/differential-report.json \
  --diff-dir artifacts/diffs
```

`--threshold` est une tolérance par canal, normalisée entre `0` et `1` puis appliquée sur `0..255`. `0` exige l’égalité exacte de chaque canal. `--max-different-ratio` autorise éventuellement une proportion de pixels dépassant cette tolérance ; sa valeur par défaut est aussi `0`. Les deux valeurs sont conservées dans le rapport JSON.

Le rapport contient `summary.static` et `summary.temporal`, puis une entrée par checkpoint/frame avec les dimensions, le nombre de pixels différents, le ratio, le delta maximal, le chemin du diff et le statut. Les chemins absents, dimensions divergentes et erreurs de décodage sont listés dans `issues` et ne sont pas transformés en succès.

Codes de sortie : `0` si toutes les images comparables passent ; `1` si la comparaison a été exécutée mais détecte une séquence, dimension ou différence de pixels ; `2` si les entrées sont invalides ou si le format PNG ne peut pas être décodé. Le rapport est écrit même dans les erreurs d’entrée lorsque le chemin `--report` est utilisable.
