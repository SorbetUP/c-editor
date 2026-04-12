void (async function () {
  if (window.__contractBootstrapStarted) {
    return;
  }

  window.__contractBootstrapStarted = true;
  window.__contractStatus = 'loading';
  window.__contractProgress = 'module-import';
  window.__contractResult = '';

  try {
    await import(new URL('../components/hybrid-note-editor.js', location.href).href);

    const params = new URLSearchParams(window.location.search);
    const fixtureName = params.get('fixture');
    const editor = document.getElementById('editor');

    if (!editor) {
      throw new Error('Missing #editor host element');
    }

    const eventLog = {
      dirtyChanges: [],
      saveRequests: [],
      changes: []
    };

    editor.addEventListener('dirty-change', (event) => {
      eventLog.dirtyChanges.push({
        dirty: Boolean(event.detail && event.detail.dirty),
        timestamp: Date.now()
      });
    });

    editor.addEventListener('save-request', (event) => {
      eventLog.saveRequests.push({
        document: event.detail ? event.detail.document : null,
        timestamp: Date.now()
      });
    });

    editor.addEventListener('change', (event) => {
      eventLog.changes.push({
        document: event.detail ? event.detail.document : null,
        timestamp: Date.now()
      });
    });

    const sleep = (ms) => new Promise((resolve) => window.setTimeout(resolve, ms));
    const waitFor = async (condition, timeoutMs, label) => {
      const startedAt = Date.now();
      while (Date.now() - startedAt < timeoutMs) {
        if (condition()) {
          return;
        }
        await sleep(50);
      }
      throw new Error(`Timeout: ${label}`);
    };

    window.__contractProgress = 'fixtures-fetch';
    const response = await fetch(new URL('./render-contract-fixtures.json', location.href).href, {
      cache: 'no-store'
    });
    if (!response.ok) {
      throw new Error(`Fixture fetch failed: ${response.status}`);
    }

    const fixturePayload = await response.json();
    const fixture = (fixturePayload.fixtures || []).find((entry) => entry.name === fixtureName);
    if (!fixture) {
      throw new Error(`Unknown fixture: ${fixtureName}`);
    }

    window.__contractProgress = 'iframe';
    await waitFor(() => editor.frame && editor.frame.contentWindow, 8000, 'iframe available');

    const frameWindow = editor.frame.contentWindow;
    window.__contractProgress = 'frame-hooks';
    await waitFor(() => Boolean(frameWindow.__hybridEditorRenderTestHooks), 15000, 'frame test hooks installed');
    await waitFor(() => Boolean(frameWindow.__hybridEditorRenderTestHooks && frameWindow.__hybridEditorRenderTestHooks.isReady()), 8000, 'frame test hooks ready');

    const frameHooks = frameWindow.__hybridEditorRenderTestHooks;
    const settle = async (delay = 140) => {
      await frameHooks.waitForSettled(delay);
      await sleep(40);
    };

    const snapshotFrameState = () => frameHooks.getState();
    const runInteractionStep = async (step) => {
      if (step.action === 'activate_line') {
        frameHooks.activateLine(step.line, step.cursor || 0);
        await settle();
        return;
      }

      if (step.action === 'set_line_markdown') {
        frameHooks.setLineMarkdown(step.line, step.markdown || '');
        await settle(520);
        return;
      }

      if (step.action === 'render_line') {
        frameHooks.renderLine(step.line);
        await settle(180);
        return;
      }

      if (step.action === 'render_all_lines') {
        frameHooks.renderAllLines();
        await settle(180);
        return;
      }

      if (step.action === 'save_request') {
        editor.saveRequest();
        await settle(180);
        return;
      }

      throw new Error(`Unsupported interaction step: ${step.action}`);
    };

    window.__contractProgress = 'load-document';
    editor.loadDocument({
      title: fixture.document_title || fixture.name,
      content: fixture.markdown
    });
    await settle(700);

    const afterLoad = snapshotFrameState();

    window.__contractProgress = 'render-all';
    frameHooks.renderAllLines();
    await settle(180);
    const renderedState = snapshotFrameState();

    window.__contractProgress = 'interactions';
    const interactions = [];
    for (const step of fixture.interaction_steps || []) {
      await runInteractionStep(step);
      interactions.push({
        action: step.action,
        line: Object.prototype.hasOwnProperty.call(step, 'line') ? step.line : null,
        state: snapshotFrameState(),
        hostDocument: editor.getDocument(),
        eventLog: {
          dirtyChanges: eventLog.dirtyChanges.slice(),
          saveRequests: eventLog.saveRequests.slice(),
          changes: eventLog.changes.slice()
        }
      });
    }

    window.__contractResult = JSON.stringify({
      version: fixturePayload.version,
      fixture: fixture.name,
      markdown: fixture.markdown,
      expected_html: fixture.expected_html,
      expected_rendered_lines: fixture.expected_rendered_lines,
      after_load: afterLoad,
      rendered_state: renderedState,
      interactions,
      engine_html: frameHooks.renderMarkdownBlock(fixture.markdown),
      debug_logs: typeof frameWindow.__getEditorDebugLogs === 'function'
        ? frameWindow.__getEditorDebugLogs()
        : [],
      host_document: editor.getDocument(),
      event_log: eventLog
    });
    window.__contractProgress = 'done';
    window.__contractStatus = 'ok';
  } catch (error) {
    window.__contractResult = JSON.stringify({
      error: error.message,
      stack: error.stack
    });
    window.__contractStatus = 'error';
    window.__contractProgress = 'failed';
  }
})();
