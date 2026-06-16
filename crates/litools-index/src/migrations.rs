use rusqlite::Connection;

use crate::schema::INITIAL_SCHEMA;

/// 执行数据库迁移。
///
/// MVP 阶段：所有表由 `INITIAL_SCHEMA` 通过 `CREATE TABLE IF NOT EXISTS` 统一创建，
/// 无需增量列迁移。
///
/// # 未来版本化迁移占位
///
/// 当首次需要修改已有表的 schema（非新增表）时，引入版本化迁移系统：
///
/// ```ignore
/// fn migrations() -> Vec<Migration> {
///     vec![
///         Migration::new(1, "initial schema", |c| c.execute_batch(INITIAL_SCHEMA)),
///         Migration::new(2, "add new_table.foo column", |c| { ... }),
///     ]
/// }
/// ```
///
/// 配合 `schema_version` 表记录当前版本，按版本号顺序执行未应用的迁移。
pub fn run_migrations(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(INITIAL_SCHEMA)?;
    Ok(())
}
