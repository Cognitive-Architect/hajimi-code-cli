// Centralized Tauri adapter for the vanilla frontend.
(function (global) {
  'use strict';

  function getGlobalInvoke() {
    const tauri = global.__TAURI__;
    if (!tauri) return null;
    const core = tauri['core'];
    return core?.['invoke'] || tauri['invoke'] || null;
  }

  function getInternalInvoke() {
    const internals = global.__TAURI_INTERNALS__;
    return internals?.invoke || null;
  }

  function getInvoke() {
    return getGlobalInvoke() || getInternalInvoke();
  }

  async function invoke(command, args = {}) {
    const rawInvoke = getInvoke();
    if (!rawInvoke) {
      throw new Error('Tauri invoke unavailable');
    }
    return rawInvoke(command, args);
  }

  function isAvailable() {
    return Boolean(getInvoke());
  }

  class Channel {
    constructor() {
      const internals = global.__TAURI_INTERNALS__;
      if (!internals?.transformCallback) {
        throw new Error('Tauri channel unavailable');
      }
      this.__TAURI_CHANNEL_MARKER__ = true;
      this._handler = () => {};
      this.id = internals.transformCallback((message) => {
        const payload = message
          && typeof message === 'object'
          && Object.prototype.hasOwnProperty.call(message, 'message')
          ? message.message
          : message;
        this._handler(payload);
      });
    }

    set onmessage(handler) {
      this._handler = typeof handler === 'function' ? handler : () => {};
    }

    get onmessage() {
      return this._handler;
    }

    toJSON() {
      return `__CHANNEL__:${this.id}`;
    }
  }

  async function listen(eventName, handler) {
    const tauri = global.__TAURI__;
    if (tauri && tauri.event && typeof tauri.event.listen === 'function') {
      return tauri.event.listen(eventName, handler);
    }
    const internals = global.__TAURI_INTERNALS__;
    if (internals && typeof internals.listen === 'function') {
      return internals.listen(eventName, handler);
    }
    if (internals && internals.event && typeof internals.event.listen === 'function') {
      return internals.event.listen(eventName, handler);
    }
    throw new Error('Tauri event listen unavailable');
  }

  global.HajimiTauri = Object.freeze({
    invoke,
    isAvailable,
    Channel,
    listen,
  });
})(window);
