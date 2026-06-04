use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PermissionDecision {
    Allow,
    Deny,
}

#[derive(Default)]
pub struct PermissionEngine {
    grants: HashMap<(String, String), PermissionDecision>,
}

impl PermissionEngine {
    pub fn grant(&mut self, plugin_id: impl Into<String>, permission: impl Into<String>) {
        self.grants.insert(
            (plugin_id.into(), permission.into()),
            PermissionDecision::Allow,
        );
    }

    pub fn deny(&mut self, plugin_id: impl Into<String>, permission: impl Into<String>) {
        self.grants.insert(
            (plugin_id.into(), permission.into()),
            PermissionDecision::Deny,
        );
    }

    pub fn check(&self, plugin_id: &str, permission: &str) -> PermissionDecision {
        self.grants
            .get(&(plugin_id.to_string(), permission.to_string()))
            .copied()
            .unwrap_or(PermissionDecision::Deny)
    }
}
