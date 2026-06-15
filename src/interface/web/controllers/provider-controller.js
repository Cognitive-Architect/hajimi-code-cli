'use strict';

(function (global) {
  function renderProviderListReadOnly(app, options) {
    return global.HajimiProviderView?.renderProviderListReadOnly?.(app, options) || {
      rendered: false,
      count: 0,
      readonly: true,
    };
  }

  const api = {
    renderProviderListReadOnly,
  };

  global.HajimiProviderController = api;

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = api;
  }
})(typeof window !== 'undefined' ? window : globalThis);
