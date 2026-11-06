import { listen } from '@tauri-apps/api/event';
import { getUpdateStatus, UPDATE_STATUS_CHANGED, type UpdateView } from '$lib/agent';

// Shared by the banner and the General tab, so both always show the same status.
let current = $state<UpdateView>();

/** The update status and setting. Reactive: kept current by NanoPrint's events. */
export function updateView(): UpdateView | undefined {
  return current;
}

export function setUpdateView(view: UpdateView): void {
  current = view;
}

/**
 * The version that waits for the user to install it: found while automatic updates are off.
 * Mirrors `waiting` in `src-tauri/src/updates.rs`, which also puts a dot on the tray icon.
 */
export function waitingVersion(view: UpdateView | undefined = current): string | undefined {
  if (!view || view.automatic) return undefined;
  const { status } = view;
  return status.state === 'available' || status.state === 'downloading' || status.state === 'ready'
    ? status.version
    : undefined;
}

/** Loads the status and follows its changes. Returns the function that stops following them. */
export function watchUpdates(onerror: (error: unknown) => void): () => void {
  getUpdateStatus().then(setUpdateView).catch(onerror);
  const unlisten = listen<UpdateView>(UPDATE_STATUS_CHANGED, (event) => {
    setUpdateView(event.payload);
  });
  return () => {
    void unlisten.then((stop) => {
      stop();
    });
  };
}
