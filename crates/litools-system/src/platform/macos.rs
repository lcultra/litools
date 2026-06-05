use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};

use plist::Value;

use crate::{DiscoveredApp, SystemAdapter};

#[derive(Default)]
pub struct MacosSystemAdapter;

impl SystemAdapter for MacosSystemAdapter {
    fn discover_apps(&self) -> Vec<DiscoveredApp> {
        discover_apps_from(default_application_dirs())
    }

    fn launch_app(&self, app_id: &str) -> Result<(), String> {
        launch_app(app_id)
    }

    fn open_file(&self, path: &str) -> Result<(), String> {
        Command::new("open")
            .arg(path)
            .status()
            .map_err(|error| error.to_string())?
            .success()
            .then_some(())
            .ok_or_else(|| format!("打开文件失败：{path}"))
    }
}

fn default_application_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from("/Applications"),
        PathBuf::from("/System/Applications"),
    ];

    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join("Applications"));
    }

    dirs
}

fn discover_apps_from(dirs: Vec<PathBuf>) -> Vec<DiscoveredApp> {
    let mut apps = Vec::new();

    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            collect_app_bundle(&path, &mut apps);

            if path.is_dir() && !is_app_bundle(&path) {
                collect_child_app_bundles(&path, &mut apps);
            }
        }
    }

    apps.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.path.cmp(&right.path))
    });
    apps.dedup_by(|left, right| left.id == right.id);
    apps
}

fn collect_child_app_bundles(dir: &Path, apps: &mut Vec<DiscoveredApp>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        collect_app_bundle(&entry.path(), apps);
    }
}

fn collect_app_bundle(path: &Path, apps: &mut Vec<DiscoveredApp>) {
    if let Some(app) = is_app_bundle(path).then(|| app_from_bundle(path)).flatten() {
        apps.push(app);
    }
}

fn is_app_bundle(path: &Path) -> bool {
    path.is_dir()
        && path
            .extension()
            .is_some_and(|extension| extension == OsStr::new("app"))
}

fn app_from_bundle(path: &Path) -> Option<DiscoveredApp> {
    let plist = read_info_plist(path);
    let name = plist
        .as_ref()
        .and_then(display_name_from_plist)
        .unwrap_or_else(|| bundle_file_name(path));
    let id = plist
        .as_ref()
        .and_then(bundle_identifier_from_plist)
        .unwrap_or_else(|| path_id(path));
    let icon_path = plist
        .as_ref()
        .and_then(|plist| icon_path_from_plist(path, plist));

    Some(DiscoveredApp {
        id,
        name,
        path: path.display().to_string(),
        icon_path,
    })
}

fn read_info_plist(bundle_path: &Path) -> Option<Value> {
    Value::from_file(bundle_path.join("Contents/Info.plist")).ok()
}

fn display_name_from_plist(plist: &Value) -> Option<String> {
    string_from_plist(plist, "CFBundleDisplayName")
        .or_else(|| string_from_plist(plist, "CFBundleName"))
}

fn bundle_identifier_from_plist(plist: &Value) -> Option<String> {
    string_from_plist(plist, "CFBundleIdentifier")
}

fn string_from_plist(plist: &Value, key: &str) -> Option<String> {
    plist
        .as_dictionary()?
        .get(key)?
        .as_string()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn bundle_file_name(path: &Path) -> String {
    path.file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or("Unknown App")
        .to_string()
}

fn path_id(path: &Path) -> String {
    let normalized = path.display().to_string();
    format!("path:{}", normalized.replace('/', ":"))
}

fn icon_path_from_plist(bundle_path: &Path, plist: &Value) -> Option<String> {
    let icon_file = string_from_plist(plist, "CFBundleIconFile")?;
    let icon_file = if Path::new(&icon_file).extension().is_some() {
        icon_file
    } else {
        format!("{icon_file}.icns")
    };
    let path = bundle_path.join("Contents/Resources").join(icon_file);

    path.exists().then(|| path.display().to_string())
}

fn launch_app(app_id: &str) -> Result<(), String> {
    let status = if app_id.starts_with('/') || app_id.starts_with("path:") {
        let path = app_id
            .strip_prefix("path:")
            .unwrap_or(app_id)
            .replace(':', "/");
        Command::new("open").arg(path).status()
    } else {
        Command::new("open").arg("-b").arg(app_id).status()
    }
    .map_err(|error| error.to_string())?;

    status
        .success()
        .then_some(())
        .ok_or_else(|| format!("打开应用失败：{app_id}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundle_file_name_removes_app_extension() {
        assert_eq!(
            bundle_file_name(Path::new("/Applications/Safari.app")),
            "Safari"
        );
    }

    #[test]
    fn path_id_is_stable_for_path() {
        assert_eq!(
            path_id(Path::new("/Applications/Example.app")),
            "path::Applications:Example.app"
        );
    }
}
