/** Behavior shared by every NanoPrint window. Call before mounting the window's component. */
export function setUpWindow(): void {
  // The webview's own menu (reload, print, save as…) does not belong in an app window. Text
  // fields and selected text keep it, for copy and paste. Development builds keep it for "Inspect".
  if (import.meta.env.PROD) {
    document.addEventListener('contextmenu', (event) => {
      const editable =
        event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement;
      const selected = (window.getSelection()?.toString() ?? '') !== '';
      if (!editable && !selected) event.preventDefault();
    });
  }
}
