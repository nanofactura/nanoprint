// Node module hooks for the compatibility tests. Bundlers let ES modules import named exports from
// qz-tray.js (a CommonJS module); Node does not, so web-escpos-printer's ES build would fail to
// load. This hands it an ES module re-exporting the very qz-tray instance the test configures.
const bridge = new URL('./qz-tray-bridge.mjs', import.meta.url).href;

export async function resolve(specifier, context, next) {
  if (specifier === 'qz-tray' && !context.parentURL?.endsWith('qz-tray-bridge.mjs')) {
    return { url: bridge, shortCircuit: true };
  }
  return next(specifier, context);
}
