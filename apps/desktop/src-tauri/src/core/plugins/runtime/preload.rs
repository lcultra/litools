pub use litools_config::events::{LIFECYCLE_ENTER_EVENT, LIFECYCLE_LEAVE_EVENT};

pub fn initialization_script() -> String {
    format!(
        r#"(function () {{
  // Transparent webview chrome + bottom rounded corners.
  var cornerStyle = document.createElement('style');
  cornerStyle.textContent = 'html {{ height: 100vh!important; width: 100vw!important; overflow: hidden!important; background: transparent!important;}} body {{ margin: 0!important; height: 100%!important; width: 100%!important; overflow: auto!important; background: transparent!important; border-radius: 0 0 20px 20px!important; }}';
  document.addEventListener('DOMContentLoaded', function () {{
    document.head.appendChild(cornerStyle);
  }});

  // Disable native context menu.
  window.addEventListener('contextmenu', function (event) {{
    event.preventDefault();
  }});

  // Lifecycle listeners.
  Object.defineProperty(window, '__litoolsLifecycleListeners', {{
    value: {{ enter: new Set(), leave: new Set() }},
    writable: false, configurable: false, enumerable: false
  }});

  function emitLifecycle(eventName) {{
    var listeners = window.__litoolsLifecycleListeners[eventName];
    if (!listeners) return;
    Array.from(listeners).forEach(function (cb) {{ try {{ cb(); }} catch (e) {{ console.error(e); }} }});
  }}

  window.addEventListener('{enter_event}', function () {{ emitLifecycle('enter'); }});
  window.addEventListener('{leave_event}', function () {{ emitLifecycle('leave'); }});
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
