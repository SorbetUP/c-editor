# Executable code blocks

## Goal

ElephantNote keeps ordinary Markdown fenced code blocks as the portable source of truth and adds optional local execution without turning notes into a proprietary notebook format.

A shared note remains readable in any Markdown application. Inside ElephantNote, the existing Muya code block owns language selection, source editing, Copy, Run/Stop and the session-local output.

## Markdown syntax

Muya persists the language in the normal fence:

````markdown
```python
print(6 * 7)
```
````

The backend recognizes Python, JavaScript/Node.js, Bash, POSIX shell, Ruby, PHP and PowerShell aliases. It only executes a language when a real interpreter is detected or configured.

## Native Muya integration

Executable controls are rendered by Muya's own fenced-code VDOM. A fence contains:

1. Muya's native language input;
2. Muya's native Copy action;
3. ElephantNote's Run/Stop action;
4. the original editable `<code>` source;
5. one `elephant-code-output` element.

All five nodes are children of the same `<pre class="ag-fence-code">`. The `<pre>` owns one border, background and radius, so source and output form one component in the note.

The runtime does not append a toolbar to `document.body`, wrap the editor, scan for code blocks or reposition controls. It has no `MutationObserver`, page-scroll listener or viewport-coordinate layout.

### Language selection

ElephantNote does not create a second language selector. The existing Muya language input and language picker remain the only source of truth. Muya's `ContentState.updateCodeLanguage()` updates the fence and performs its normal partial render.

Editing the language is transactional. Keystrokes update only the visible draft and the picker; the Markdown fence is committed once when the user chooses a suggestion, presses Enter or leaves the field. Deleting `python` therefore no longer produces six separate note saves for `pytho`, `pyth`, `pyt`, and so on.

The picker prioritizes exact and prefix matches, removes irrelevant one-letter fuzzy aliases, limits the result list to eight entries and shows readable language titles with aliases when useful.

This avoids synthetic `input` and `change` events. In particular, changing a language cannot recursively redispatch an event onto the same selector or run Muya's keyboard input handler with an unrelated selection.

## Runtime lifecycle

`elephant-code-output` is a custom element rendered by Muya. Its connection lifecycle registers the corresponding fence with the execution runtime.

State is keyed by editor root and Muya block key. When Muya partially rerenders a fence:

- the old output element disconnects;
- the replacement connects with the same block key;
- the existing result and running state are reused;
- deletion is pruned after a short grace period.

No DOM topology scan is required.

## Running and stopping

Clicking Run reads language and source directly from the same native `<pre>`, then invokes `tauri_programs_run` through Tauri. `Cmd+Enter` on macOS and `Ctrl+Enter` elsewhere use the same path.

Each execution has a unique identifier. While running, the triangle becomes a square Stop control. Stop targets that exact process.

On Unix, the backend sends `SIGINT` first, waits briefly for a graceful exit, then force-terminates and reaps a process that ignores the interrupt. Other platforms use the supported termination fallback.

## Output

Output is session-local and is never written into Markdown. The custom element uses a shadow root so Muya's VDOM can rerender the fence without serializing or patching the output contents.

The output section:

- expands below the source inside the same `<pre>`;
- inherits the note theme;
- separates stdout and stderr;
- scrolls internally when long;
- supports Copy, Collapse/Expand and Clear;
- shows duration, exit status, timeout, interruption and truncation.

## Bounded execution

The Rust backend never collects unbounded process output. stdout and stderr are drained concurrently into fixed-size tail buffers.

- source is limited to 256 KiB;
- each stream retains at most 1 MiB;
- only the configured final lines are returned;
- execution has a timeout;
- interruption and timeout both kill and reap the child process;
- interpreters are spawned directly, not through a generic shell;
- the working directory is constrained to the active vault.

## Security model

This implementation is **not a sandbox or container**. A program runs with the same operating-system user permissions as ElephantNote. Code may access user-readable files, use the network, launch subprocesses or modify the machine.

## Validation

The active tests cover both isolated runtime behavior and a real Muya instance. They verify that:

- Run and Output are native children of the fenced `<pre>`;
- installing the runtime changes zero Markdown characters;
- no V6 toolbar or duplicate language selector is created;
- partial language typing does not save or alter Markdown;
- a completed language edit emits one change only;
- `py` returns relevant, bounded suggestions instead of unrelated one-letter aliases;
- the native Muya language state is used for execution;
- repeated language changes do not recurse or emit synthetic keyboard input;
- real Run and Stop IPC requests are sent;
- output survives a Muya rerender with the same block key;
- deletion prunes state without a mutation scan;
- multiple blocks keep independent states;
- page and output scrolling trigger no runtime work;
- stream capture remains bounded;
- a real Python process can execute and be interrupted when Python is available.
