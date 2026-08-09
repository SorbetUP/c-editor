# BUG: NoteEditorHost se démontait lors d’un rafraîchissement du vault

## Symptôme

Les logs montraient `before unmount` avec `notePath:""`, suivi moins d’une seconde plus tard par un nouveau `mounted` de la même note. Le processus terminait avec `code:0`, mais ce cycle pouvait interrompre Muya Rust et perdre son état visuel.

## Cause

`vaultStore.applyPayload()` vidait toujours `openedNotePath` et `openedNotes`. Un payload de rafraîchissement du même vault était donc traité comme une fermeture de note.

## Correction

Le store conserve la note ouverte et les notes récentes quand `activeVaultId` reste identique. Lors d’un changement réel de vault, le comportement de nettoyage reste inchangé. Le log `[vault] applyPayload` indique désormais `preservesActiveNote` et le chemin concerné.

## Vérification

Le test `vaultStoreRealWorkflow.spec.js` couvre le rafraîchissement du payload : **12 tests passés**. ESLint ciblé et `git diff --check` passent.
