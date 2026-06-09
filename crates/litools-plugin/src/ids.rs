pub use litools_config::plugin::{PLUGIN_TARGET_TYPE};
pub use litools_config::search::{PLUGIN_RESULT_PREFIX};

/// Build a search result ID from a plugin ID and command ID.
///
/// Format: `plugin:{plugin_id}:{command_id}`
pub fn plugin_result_id(plugin_id: &str, command_id: &str) -> String {
    format!("{PLUGIN_RESULT_PREFIX}{plugin_id}:{command_id}")
}

/// Build a target ID from a plugin ID and command ID.
///
/// Format: `{plugin_id}:{command_id}`
pub fn plugin_target_id(plugin_id: &str, command_id: &str) -> String {
    format!("{plugin_id}:{command_id}")
}

/// Parse a result ID back into (plugin_id, command_id).
pub fn plugin_command_from_result_id(result_id: &str) -> Option<(&str, &str)> {
    let rest = result_id.strip_prefix(PLUGIN_RESULT_PREFIX)?;
    let (plugin_id, command_id) = rest.rsplit_once(':')?;
    if plugin_id.is_empty() || command_id.is_empty() {
        return None;
    }
    Some((plugin_id, command_id))
}

/// Parse a target ID back into (plugin_id, command_id).
pub fn plugin_command_from_target_id(target_id: &str) -> Option<(&str, &str)> {
    let (plugin_id, command_id) = target_id.rsplit_once(':')?;
    if plugin_id.is_empty() || command_id.is_empty() {
        return None;
    }
    Some((plugin_id, command_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plugin_result_id() {
        assert_eq!(
            plugin_command_from_result_id("plugin:dev.litools.hello-world:hello"),
            Some(("dev.litools.hello-world", "hello"))
        );
    }

    #[test]
    fn parses_plugin_target_id() {
        assert_eq!(
            plugin_command_from_target_id("dev.litools.hello-world:hello"),
            Some(("dev.litools.hello-world", "hello"))
        );
    }

    #[test]
    fn round_trips_result_id() {
        let result_id = plugin_result_id("dev.litools.foo", "bar");
        let (plugin_id, command_id) =
            plugin_command_from_result_id(&result_id).expect("parse result id");
        assert_eq!(plugin_id, "dev.litools.foo");
        assert_eq!(command_id, "bar");
    }

    #[test]
    fn round_trips_target_id() {
        let target_id = plugin_target_id("dev.litools.foo", "bar");
        let (plugin_id, command_id) =
            plugin_command_from_target_id(&target_id).expect("parse target id");
        assert_eq!(plugin_id, "dev.litools.foo");
        assert_eq!(command_id, "bar");
    }
}
