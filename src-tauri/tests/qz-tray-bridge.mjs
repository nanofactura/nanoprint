// qz-tray.js as an ES module with named exports (see qz-tray-esm.mjs).
import qz from 'qz-tray';

export default qz;
export const { api, configs, print, printers, security, websocket } = qz;
