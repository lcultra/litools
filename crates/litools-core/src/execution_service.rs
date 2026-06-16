use chrono::Utc;
use litools_index::{
    IndexDatabase,
    repository::{AppRepository, UsageRepository},
};
use litools_plugin::{PluginCommandMode, PluginManager, plugin_target_id};
use litools_system::SystemAdapter;
use uuid::Uuid;

use litools_config::search::ACTION_OPEN;

use crate::{
    command::{CommandEffect, CommandExecution},
    error::{LitoolsError, LitoolsResult},
};

/// 执行应用的"打开"操作：启动应用 → 更新启动计数 → 记录使用历史。
pub fn execute_app(
    database: &IndexDatabase,
    system_adapter: &dyn SystemAdapter,
    result_id: &str,
    app_id: &str,
    action_id: &str,
) -> LitoolsResult<CommandExecution> {
    if action_id != ACTION_OPEN {
        return Err(LitoolsError::CommandNotFound(format!(
            "{result_id}:{action_id}"
        )));
    }

    let connection = database.connection();
    let apps = AppRepository::new(&connection);
    let app = apps
        .find_app(app_id)?
        .ok_or_else(|| LitoolsError::AppNotFound(result_id.to_string()))?;

    system_adapter
        .launch_app(&app.id)
        .or_else(|_| system_adapter.launch_app(&app.path))
        .map_err(LitoolsError::System)?;
    apps.increment_launch_count(&app.id)?;
    UsageRepository::new(&connection).record_selection(
        &Uuid::new_v4().to_string(),
        "app",
        &app.id,
        None,
        &Utc::now().to_rfc3339(),
    )?;

    Ok(CommandExecution {
        message: format!("正在打开 {}", app.name),
        result_id: result_id.to_string(),
        action_id: action_id.to_string(),
        effect: CommandEffect::None,
    })
}

/// 执行插件命令的"打开"操作：验证插件/命令 → 记录使用历史 → 返回 OpenPluginView effect。
pub fn execute_plugin(
    plugins: &PluginManager,
    database: &IndexDatabase,
    result_id: &str,
    plugin_id: &str,
    command_id: &str,
    action_id: &str,
) -> LitoolsResult<CommandExecution> {
    if action_id != ACTION_OPEN {
        return Err(LitoolsError::CommandNotFound(format!(
            "{result_id}:{action_id}"
        )));
    }

    let plugin = plugins
        .find_plugin(plugin_id)
        .ok_or_else(|| LitoolsError::PluginNotFound(result_id.to_string()))?;
    if !plugin.enabled {
        return Err(LitoolsError::PluginDisabled(result_id.to_string()));
    }
    let command = plugin
        .manifest
        .commands
        .iter()
        .find(|command| command.id == command_id)
        .ok_or_else(|| LitoolsError::CommandNotFound(result_id.to_string()))?;
    if command.mode != PluginCommandMode::View {
        return Err(LitoolsError::InvalidAction(format!(
            "plugin command is not a view: {result_id}"
        )));
    }

    let route = plugin_route(plugin_id, command_id);
    database.read(|conn| {
        UsageRepository::new(&conn).record_selection(
            &Uuid::new_v4().to_string(),
            litools_plugin::PLUGIN_TARGET_TYPE,
            &plugin_target_id(plugin_id, command_id),
            None,
            &Utc::now().to_rfc3339(),
        )
    })?;

    Ok(CommandExecution {
        message: format!("正在打开 {}", command.title),
        result_id: result_id.to_string(),
        action_id: action_id.to_string(),
        effect: CommandEffect::OpenPluginView {
            plugin_id: plugin_id.to_string(),
            command_id: command_id.to_string(),
            route,
        },
    })
}

/// 记录内置命令的使用历史。
pub fn record_builtin_usage(database: &IndexDatabase, command_id: &str) -> LitoolsResult<()> {
    database.read(|conn| {
        UsageRepository::new(&conn).record_selection(
            &Uuid::new_v4().to_string(),
            "command",
            command_id,
            None,
            &Utc::now().to_rfc3339(),
        )
    })?;
    Ok(())
}

fn plugin_route(plugin_id: &str, command_id: &str) -> String {
    format!("/plugin/{plugin_id}/{command_id}")
}
