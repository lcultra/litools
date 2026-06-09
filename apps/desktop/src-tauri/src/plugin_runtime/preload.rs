pub const LIFECYCLE_ENTER_EVENT: &str = "plugin-runtime://enter";
pub const LIFECYCLE_LEAVE_EVENT: &str = "plugin-runtime://leave";

pub fn initialization_script() -> String {
    format!(
        r#"(function () {{
  if (Object.prototype.hasOwnProperty.call(window, 'litools')) {{
    return;
  }}

  // Transparent webview chrome + bottom rounded corners matching Panel rounded-[20px].
  // html locks viewport and hides overflow; body gets an internal scroll context
  // with rounded bottom corners. The clipped corner area is transparent, revealing
  // the Panel bg-surface and border underneath.
  var cornerStyle = document.createElement('style');
  cornerStyle.textContent = 'html {{ height: 100vh!important; width: 100vw!important; overflow: hidden!important;  background: transparent!important;}} body {{ margin: 0!important; height: 100%!important; width: 100%!important; overflow: auto!important; background: transparent!important; border-radius: 0 0 20px 20px!important; }}';
  document.addEventListener('DOMContentLoaded', function () {{
    document.head.appendChild(cornerStyle);
  }});

  // Disable native context menu on all plugin webviews.
  window.addEventListener('contextmenu', function (event) {{
    event.preventDefault();
  }});

  function invokeRuntime(method, params) {{
    var internals = window.__TAURI_INTERNALS__;
    if (!internals || typeof internals.invoke !== 'function') {{
      return Promise.reject(new Error('litools runtime bridge is unavailable'));
    }}
    return internals.invoke('plugin_runtime_call', {{ method: method, params: params || {{}} }});
  }}

  function listenRuntimeEvent(eventName, callback) {{
    if (typeof callback !== 'function') {{
      throw new TypeError('callback must be a function');
    }}
    window.__litoolsLifecycleListeners[eventName].add(callback);
    return function unsubscribe() {{
      window.__litoolsLifecycleListeners[eventName].delete(callback);
    }};
  }}

  Object.defineProperty(window, '__litoolsLifecycleListeners', {{
    value: {{
      enter: new Set(),
      leave: new Set()
    }},
    writable: false,
    configurable: false,
    enumerable: false
  }});

  function emitLifecycle(eventName) {{
    var listeners = window.__litoolsLifecycleListeners[eventName];
    if (!listeners) {{
      return;
    }}
    Array.from(listeners).forEach(function (listener) {{
      try {{
        listener();
      }} catch (error) {{
        console.error(error);
      }}
    }});
  }}

  window.addEventListener('{enter_event}', function () {{ emitLifecycle('enter'); }});
  window.addEventListener('{leave_event}', function () {{ emitLifecycle('leave'); }});

  var runtime = Object.freeze({{
    ready: function () {{ return invokeRuntime('runtime.ready'); }},
    getInfo: function () {{ return invokeRuntime('runtime.getInfo'); }}
  }});
  var permissions = Object.freeze({{
    query: function (permission) {{ return invokeRuntime('permissions.query', {{ permission: permission }}); }}
  }});
  var lifecycle = Object.freeze({{
    onEnter: function (callback) {{ return listenRuntimeEvent('enter', callback); }},
    onLeave: function (callback) {{ return listenRuntimeEvent('leave', callback); }}
  }});
  var ui = Object.freeze({{
    close: function () {{ return invokeRuntime('ui.close'); }},
    setTitle: function (title) {{ return invokeRuntime('ui.setTitle', {{ title: title }}); }},
    toast: function (message, options) {{ return invokeRuntime('ui.toast', {{ message: message, options: options || null }}); }}
  }});
  var storage = Object.freeze({{
    get: function (key) {{ return invokeRuntime('storage.get', {{ key: key }}); }},
    set: function (key, value) {{ return invokeRuntime('storage.set', {{ key: key, value: value }}); }},
    remove: function (key) {{ return invokeRuntime('storage.remove', {{ key: key }}); }},
    clear: function () {{ return invokeRuntime('storage.clear'); }}
  }});
  var settings = Object.freeze({{
    get: function () {{ return invokeRuntime('settings.get'); }},
    update: function (settings) {{ return invokeRuntime('settings.update', {{ settings: settings }}); }}
  }});
  var diagnostics = Object.freeze({{
    get: function () {{ return invokeRuntime('diagnostics.get'); }}
  }});
  var plugins = Object.freeze({{
    list: function () {{ return invokeRuntime('plugins.list'); }}
  }});

  var litools = Object.freeze({{
    runtime: runtime,
    permissions: permissions,
    lifecycle: lifecycle,
    ui: ui,
    storage: storage,
    settings: settings,
    diagnostics: diagnostics,
    plugins: plugins
  }});

  Object.defineProperty(window, 'litools', {{
    value: litools,
    writable: false,
    configurable: false,
    enumerable: true
  }});
}})();"#,
        enter_event = LIFECYCLE_ENTER_EVENT,
        leave_event = LIFECYCLE_LEAVE_EVENT,
    )
}

pub fn lifecycle_eval_script(event_name: &str) -> String {
    format!(
        "window.dispatchEvent(new CustomEvent({}));",
        serde_json::to_string(event_name).expect("event name serializes")
    )
}
