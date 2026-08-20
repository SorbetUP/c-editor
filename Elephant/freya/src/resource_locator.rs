//! Stable resource discovery for the Freya development shell.
//!
//! Development builds may resolve resources from the repository checkout, but
//! release builds must resolve them relative to the executable/bundle. This
//! prevents sidecars and runtimes from depending on the process working
//! directory.

use std::{
    env,
    path::{Path, PathBuf},
};

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

    runtime_binary("node", "node", "ELEPHANT_FREYA_ALLOW_SYSTEM_NODE")
        .or_else(|error| {
            if cfg!(debug_assertions) {
                Ok(PathBuf::from(platform_executable_name("node")))
            } else {
                Err(error)
            }
        })
}

/// Resolve the package-owned llama.cpp server used by Open Models.
///
/// The official Open Models service already honours `ELEPHANT_LLAMA_SERVER_PATH`.
/// Freya supplies that variable when the runtime is bundled, while development
/// may explicitly opt into a system `llama-server` binary.
pub fn llama_server() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("ELEPHANT_LLAMA_SERVER_PATH") {
        let path = PathBuf::from(path);
        return existing_file(path).map_err(|path| {
            format!(
                "ELEPHANT_LLAMA_SERVER_PATH does not point to a file: {}",
                path.display()
            )
        });
    }

    runtime_binary(
        "llama",
        "llama-server",
        "ELEPHANT_FREYA_ALLOW_SYSTEM_LLAMA",
    )
    .or_else(|error| {
        if cfg!(debug_assertions)
            || env::var_os("ELEPHANT_FREYA_ALLOW_SYSTEM_LLAMA").as_deref()
                == Some(std::ffi::OsStr::new("1"))
        {
            Ok(PathBuf::from(platform_executable_name("llama-server")))
        } else {
            Err(error)
        }
    })
}

fn runtime_binary(
    runtime_directory: &str,
    executable_name: &str,
    allow_system_env: &str,
) -> Result<PathBuf, String> {
    let platform = NativePlatform::current()?.directory();
    let file = platform_executable_name(executable_name);
    for root in installed_resource_roots() {
        for relative in [
            PathBuf::from("runtimes")
                .join(runtime_directory)
                .join(platform)
                .join(&file),
            PathBuf::from("runtime")
                .join(runtime_directory)
                .join(platform)
                .join(&file),
        ] {
            let candidate = root.join(relative);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }

    if env::var_os(allow_system_env).as_deref() == Some(std::ffi::OsStr::new("1")) {
        return Ok(PathBuf::from(file));
    }

    Err(format!(
        "No packaged {executable_name} runtime is available for {platform}"
    ))
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

    #[test]
    fn explicit_llama_override_must_be_a_file() {
        let previous = env::var_os("ELEPHANT_LLAMA_SERVER_PATH");
        env::set_var("ELEPHANT_LLAMA_SERVER_PATH", "/definitely/missing/llama-server");
        assert!(llama_server().is_err());
        match previous {
            Some(value) => env::set_var("ELEPHANT_LLAMA_SERVER_PATH", value),
            None => env::remove_var("ELEPHANT_LLAMA_SERVER_PATH"),
        }
    }
}
