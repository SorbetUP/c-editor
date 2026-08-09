# Addon Platform Proof

This addon is a deterministic end-to-end proof of the external ElephantNote addon contract. It does not use a network service.

The command exercises:

1. package installation into the active vault;
2. isolated JavaScript Worker activation;
3. command registration and execution;
4. the brokered `app.info` API;
5. private addon storage read/write;
6. scoped note write under `Addon Proof/**`;
7. scoped note read-back;
8. result propagation back to ElephantNote;
9. vault refresh and opening the generated note.

## Package

From the repository root:

```bash
node build/scripts/package-addon.mjs \
  examples/addons/platform-proof \
  build/addons/platform-proof.enaddon
```

Then open **Settings → Addons**, turn on Community Addons, select **Install**, and choose:

```text
build/addons/platform-proof.enaddon
```

Enable **Addon Platform Proof**, expand it, then run **Run addon platform proof**.

A successful run opens:

```text
Addon Proof/External Addon Platform Proof.md
```

The note contains a unique `ELEPHANT_ADDON_PROOF` marker and the stored run counter. Running it a second time must increment the counter, proving that private addon storage is persistent rather than mocked in memory.
