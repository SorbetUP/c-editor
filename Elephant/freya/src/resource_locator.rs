//! Stable resource discovery for the Freya development shell.
//!
//! Development builds may resolve resources from the repository checkout, but
//! release builds must resolve them relative to the executable/bundle. This
//! prevents sidecars and runtimes from depending on the process working
//! directory.

use std::{env, path::{Path, PathBuf}};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativePlatform {
    MacosAarch64,
    MacosX8664,
    LinuxX8664,
    WindowsX8664,
}

impl NativePlatform {
    pub fn current() -> Result<Self, String> {
        if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
            Ok(Self::MacosAarch64)
        } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
            Ok(Self::MacosX8664)
        } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            Ok(Self::LinuxX8664)
        } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
            Ok(Self::WindowsX8664)
        } else {
            Err(format!(
                "No packaged Elephant native runtime is defined for {}-{}",
                env::consts::OS,
                env::consts::ARCH
            ))
        }
    }

    pub fn directory(self) -> &'static str {
        match self {
            Self::MacosAarch64 => "macos-aarch64",
            Self::MacosX8664 => "macos-x86_64",
            Self::LinuxX8664 => "linux-x86_64",
            Self::WindowsX8664 => "windows-x86_64",
        }
    }
}

pub fn native_service(
    env_override: &str,
    addon_directory: &str,
    executable_name: &str,
) -> Result<PathBuf, String> {
    if let Some(path) = env::var_os(env_override) {
        let path = PathBuf::from(path);
        return existing_file(path).map_err(|path| {
            format!("{env_override} does not point to a file: {}", path.display())
        });
    }

    let platform = NativePlatform::current()?.directory();
    let file_name = platform_executable_name(executable_name);
    let relative = PathBuf::from("official-addons")
        .join("official")
        .join(addon_directory)
        .join("native")
        .join(platform)
        .join(file_name);

    for root in installed_resource_roots() {
        let candidate = root.join(&relative);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    // Repository lookup is intentionally development-only. It keeps `cargo
    // run` and integration tests convenient without making release artifacts
    // dependent on the source checkout.
    if cfg!(debug_assertions) {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../addons")
            .join("official")
            .join(addon_directory)
            .join("native")
            .join(platform)
            .join(platform_executable_name(executable_name));
        if source.is_file() {
            return Ok(source);
        }
    }

    Err(format!(
        "Packaged Elephant {addon_directory} service executable was not found"
    ))
}

pub fn node_runtime() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("ELEPHANT_NODE_RUNTIME") {
        let path = PathBuf::from(path);
        return existing_file(path).map_err(|path| {
            format!(
                "ELEPHANT_NODE_RUNTIME does not point to a file: {}",
                path.display()
            )
        });
    }

    let platform = NativePlatform::current()?.directory();
    let file = platform_executable_name("node");
    for root in installed_resource_roots() {
        for relative in [
            PathBuf::from("runtimes").join("node").join(platform).join(&file),
            PathBuf::from("runtime").join("node").join(platform).join(&file),
        ] {
            let candidate = root.join(relative);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }

    // System Node remains a developer/test convenience only. A packaged
    // release must either bundle Node or opt in explicitly.
    if cfg!(debug_assertions)
        || env::var_os("ELEPHANT_FREYA_ALLOW_SYSTEM_NODE").as_deref() == Some(std::ffi::OsStr::new("1"))
    {
        return Ok(PathBuf::from(platform_executable_name("node")));
    }

    Err("No packaged Node runtime is available for external addons".to_owned())
}

pub fn installed_resource_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Some(root) = env::var_os("ELEPHANT_RESOURCE_DIR") {
        push_unique(&mut roots, PathBuf::from(root));
    }

    if let Some(appdir) = env::var_os("APPDIR") {
        let appdir = PathBuf::from(appdir);
        push_unique(&mut roots, appdir.join("usr/lib/elephant/resources"));
        push_unique(&mut roots, appdir.join("usr/share/elephant/resources"));
    }

    if let Ok(executable) = env::current_exe() {
        if let Some(directory) = executable.parent() {
            push_unique(&mut roots, directory.join("resources"));
            push_unique(&mut roots, directory.join("../Resources"));
            push_unique(&mut roots, directory.join("../share/elephant/resources"));
            push_unique(&mut roots, directory.join("../lib/elephant/resources"));
        }
    }

    roots
}

fn platform_executable_name(base: &str) -> String {
    if cfg!(target_os = "windows") && !base.to_ascii_lowercase().ends_with(".exe") {
        format!("{base}.exe")
    } else {
        base.to_owned()
    }
}

fn existing_file(path: PathBuf) -> Result<PathBuf, PathBuf> {
    if path.is_file() {
        Ok(path)
    } else {
        Err(path)
    }
}

fn push_unique(roots: &mut Vec<PathBuf>, candidate: PathBuf) {
    if !roots.iter().any(|root| same_path(root, &candidate)) {
        roots.push(candidate);
    }
}

fn same_path(left: &Path, right: &Path) -> bool {
    left == right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_directory_is_not_empty_on_supported_ci_platforms() {
        if let Ok(platform) = NativePlatform::current() {
            assert!(!platform.directory().is_empty());
        }
    }

    #[test]
    fn installed_roots_do_not_depend_on_current_directory() {
        let roots = installed_resource_roots();
        let current = env::current_dir().unwrap();
        assert!(roots.iter().all(|root| root != &current.join("resources")));
    }
}