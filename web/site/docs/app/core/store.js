export function createStore(initialState) {
  let state = initialState;
  const listeners = new Set();

  return {
    getState() {
      return state;
    },
    subscribe(listener) {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    setState(updater) {
      state = typeof updater === 'function' ? updater(state) : updater;
      listeners.forEach((listener) => listener(state));
    }
  };
}
