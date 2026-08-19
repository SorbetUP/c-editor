//! Permissioned host RPC implementation for Freya JavaScript addons.
//!
//! The worker intentionally has no direct filesystem, network, Tauri or Node
//! capability. Every privileged operation crosses this module and is checked
//! against the persisted addon manifest before touching the active vault.

use crate::{addon_adapter::InstalledAddon, vault_layout};
use reqwest::{
    blocking::{Client, Response},
    header::LOCATION,
    Method,
};
use serde_json::{json, Map, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::{ErrorKind, Read, Write},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs},
    path::{Component, Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use url::Url;

type Result<T> = std::result::Result<T, String>;

const MAX_DIRECTORY_DEPTH: usize = 64;
const MAX_LISTED_NOTES: usize = 1_000;
const MAX_NOTE_BYTES: u64 = 5 * 1024 * 1024;
const MAX_STORAGE_BYTES: usize = 2 * 1024 * 1024;
const MAX_HTTP_REQUEST_BYTES: usize = 1024 * 1024;
const MAX_HTTP_RESPONSE_BYTES: u64 = 5 * 1024 * 1024;
const MAX_REDIRECTS: usize = 5;

pub(crate) fn handle(
    root: &Path,
    addon: &InstalledAddon,
    method: &str,
    params: &Value,
) -> Result<Value> {
    match method {
        "app.info" => Ok(json!({
            "name": "ElephantNote",
            "runtime": "freya",
            "version": env!("CARGO_PKG_VERSION"),
            "addonApiVersion": crate::addon_adapter::ADDON_API_VERSION
        })),
        "storage.get" => storage_get(root, addon, params),
        "storage.set" => storage_set(root, addon, params),
        "storage.remove" => storage_remove(root, addon, params),
        "storage.entries" => storage_entries(root, addon),
        "notes.list" => notes_list(root, addon, params),
        "notes.read" => notes_read(root, addon, params),
        "notes.write" => notes_write(root, addon, params),
        "http.request" => http_request(addon, params),
        _ => Err(format!("Freya addon host RPC is unavailable: {method}")),
    }
}

fn permissions(addon: &InstalledAddon) -> &Value {
    addon
        .manifest
        .extra
        .get("permissions")
        .unwrap_or(&Value::Null)
}

fn permission_strings(addon: &InstalledAddon, pointer: &str) -> Vec<String> {
    permissions(addon)
        .pointer(pointer)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}

fn scope_matches(scope: &str, relative_path: &str) -> bool {
    let scope = scope.trim().replace('\\', "/");
    if scope == "*" {
        return true;
    }
    if let Some(prefix) = scope.strip_suffix("/**") {
        let prefix = prefix.trim_end_matches('/');
        return relative_path == prefix || relative_path.starts_with(&format!("{prefix}/"));
    }
    relative_path == scope
}

fn normalize_relative_path(value: &str, empty_error: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(empty_error.to_owned());
    }
    let path = Path::new(value);
    if path.is_absolute() {
        return Err("Addon paths must be relative to the active vault".to_owned());
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(format!("Path traversal is not allowed: {value}"));
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err(empty_error.to_owned());
    }
    Ok(normalized.to_string_lossy().replace('\\', "/"))
}

fn canonical_root(root: &Path) -> Result<PathBuf> {
    let root = fs::canonicalize(root).map_err(|error| format!("Open active vault: {error}"))?;
    if !root.is_dir() {
        return Err("Active vault root is not a directory".to_owned());
    }
    Ok(root)
}

fn addon_data_dir(root: &Path, addon: &InstalledAddon) -> Result<PathBuf> {
    let root = canonical_root(root)?;
    let data = vault_layout::addons_dir(&root)
        .join("data")
        .join(&addon.manifest.id);
    fs::create_dir_all(&data).map_err(|error| format!("Create addon data directory: {error}"))?;
    let data = fs::canonicalize(&data).map_err(|error| error.to_string())?;
    if !data.starts_with(&root) {
        return Err("Addon data directory escaped the active vault".to_owned());
    }
    Ok(data)
}

fn storage_path(root: &Path, addon: &InstalledAddon) -> Result<PathBuf> {
    Ok(addon_data_dir(root, addon)?.join("storage.json"))
}

fn storage_key(params: &Value) -> Result<String> {
    let key = params.get("key").and_then(Value::as_str).unwrap_or("").trim();
    if key.is_empty() || key.len() > 256 || key.chars().any(char::is_control) {
        return Err("Addon storage key is invalid".to_owned());
    }
    Ok(key.to_owned())
}

fn read_storage(root: &Path, addon: &InstalledAddon) -> Result<BTreeMap<String, Value>> {
    let path = storage_path(root, addon)?;
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let metadata = fs::metadata(&path).map_err(|error| error.to_string())?;
    if metadata.len() as usize > MAX_STORAGE_BYTES {
        return Err("Addon storage exceeds the 2 MiB limit".to_owned());
    }
    let raw = fs::read(&path).map_err(|error| error.to_string())?;
    serde_json::from_slice(&raw).map_err(|error| format!("Invalid addon storage: {error}"))
}

fn write_storage(
    root: &Path,
    addon: &InstalledAddon,
    storage: &BTreeMap<String, Value>,
) -> Result<()> {
    let path = storage_path(root, addon)?;
    let bytes = serde_json::to_vec_pretty(storage).map_err(|error| error.to_string())?;
    if bytes.len() > MAX_STORAGE_BYTES {
        return Err("Addon storage exceeds the 2 MiB limit".to_owned());
    }
    atomic_write(&path, &bytes)
}

fn storage_get(root: &Path, addon: &InstalledAddon, params: &Value) -> Result<Value> {
    let key = storage_key(params)?;
    Ok(read_storage(root, addon)?.remove(&key).unwrap_or(Value::Null))
}

fn storage_set(root: &Path, addon: &InstalledAddon, params: &Value) -> Result<Value> {
    let key = storage_key(params)?;
    let value = params.get("value").cloned().unwrap_or(Value::Null);
    let mut storage = read_storage(root, addon)?;
    storage.insert(key, value);
    write_storage(root, addon, &storage)?;
    Ok(Value::Bool(true))
}

fn storage_remove(root: &Path, addon: &InstalledAddon, params: &Value) -> Result<Value> {
    let key = storage_key(params)?;
    let mut storage = read_storage(root, addon)?;
    let removed = storage.remove(&key).is_some();
    write_storage(root, addon, &storage)?;
    Ok(Value::Bool(removed))
}

fn storage_entries(root: &Path, addon: &InstalledAddon) -> Result<Value> {
    serde_json::to_value(read_storage(root, addon)?).map_err(|error| error.to_string())
}

fn normalize_listing_prefix(value: &str) -> Result<String> {
    if value.trim().is_empty() || value.trim() == "." {
        return Ok(String::new());
    }
    normalize_relative_path(value, "A note directory or '.' for the vault root is required")
}

fn is_hidden_component(path: &Path) -> bool {
    path.components().any(|component| match component {
        Component::Normal(part) => part.to_string_lossy().starts_with('.'),
        _ => false,
    })
}

fn validate_markdown_path(value: &str) -> Result<String> {
    let relative_path = normalize_relative_path(value, "A note path is required")?;
    let relative = Path::new(&relative_path);
    if is_hidden_component(relative) {
        return Err("Addons cannot access notes in hidden directories".to_owned());
    }
    if relative
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.eq_ignore_ascii_case("md"))
        != Some(true)
    {
        return Err("Addon note access is limited to Markdown files".to_owned());
    }
    Ok(relative_path)
}

fn note_read_scopes(addon: &InstalledAddon) -> Vec<String> {
    permission_strings(addon, "/notes/read")
}

fn note_write_scopes(addon: &InstalledAddon) -> Vec<String> {
    permission_strings(addon, "/notes/write")
}

fn permitted(scopes: &[String], path: &str) -> bool {
    !scopes.is_empty() && scopes.iter().any(|scope| scope_matches(scope, path))
}

fn modified_at(metadata: &fs::Metadata) -> Option<u64> {
    metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
}

fn notes_list(root: &Path, addon: &InstalledAddon, params: &Value) -> Result<Value> {
    let prefix = normalize_listing_prefix(params.get("prefix").and_then(Value::as_str).unwrap_or("."))?;
    let scopes = note_read_scopes(addon);
    let can_list = if prefix.is_empty() {
        scopes.iter().any(|scope| scope == "*")
    } else {
        permitted(&scopes, &prefix)
    };
    if !can_list {
        return Err(format!(
            "Addon is not permitted to list notes under {}",
            if prefix.is_empty() { "the vault root" } else { &prefix }
        ));
    }
    let root = canonical_root(root)?;
    let start = if prefix.is_empty() { root.clone() } else { root.join(&prefix) };
    if !start.exists() {
        return Ok(Value::Array(Vec::new()));
    }
    let start = fs::canonicalize(start).map_err(|error| error.to_string())?;
    if !start.starts_with(&root) {
        return Err("Refusing to list notes outside the active vault".to_owned());
    }
    let mut stack = vec![(start, 0usize)];
    let mut seen = BTreeSet::new();
    let mut notes = Vec::new();
    while let Some((directory, depth)) = stack.pop() {
        if depth > MAX_DIRECTORY_DEPTH {
            return Err(format!("Addon note listing exceeded depth {MAX_DIRECTORY_DEPTH}"));
        }
        for entry in fs::read_dir(&directory).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let file_type = entry.file_type().map_err(|error| error.to_string())?;
            if file_type.is_symlink() {
                continue;
            }
            let path = entry.path();
            let relative = path
                .strip_prefix(&root)
                .map_err(|_| "Listed note escaped the active vault".to_owned())?;
            if is_hidden_component(relative) {
                continue;
            }
            let relative_path = relative.to_string_lossy().replace('\\', "/");
            if file_type.is_dir() {
                stack.push((path, depth + 1));
                continue;
            }
            if !file_type.is_file()
                || path.extension().and_then(|value| value.to_str()).map(|value| value.eq_ignore_ascii_case("md")) != Some(true)
                || !permitted(&scopes, &relative_path)
                || !seen.insert(relative_path.clone())
            {
                continue;
            }
            if notes.len() >= MAX_LISTED_NOTES {
                return Err(format!("Addon note listing exceeded {MAX_LISTED_NOTES} notes"));
            }
            let metadata = entry.metadata().map_err(|error| error.to_string())?;
            notes.push(json!({
                "path": relative_path,
                "size": metadata.len(),
                "modifiedAt": modified_at(&metadata)
            }));
        }
    }
    notes.sort_by(|left, right| left["path"].as_str().cmp(&right["path"].as_str()));
    Ok(Value::Array(notes))
}

fn notes_read(root: &Path, addon: &InstalledAddon, params: &Value) -> Result<Value> {
    let relative_path = validate_markdown_path(params.get("path").and_then(Value::as_str).unwrap_or(""))?;
    let scopes = note_read_scopes(addon);
    if !permitted(&scopes, &relative_path) {
        return Err(format!("Addon is not permitted to read {relative_path}"));
    }
    let root = canonical_root(root)?;
    let target = fs::canonicalize(root.join(&relative_path))
        .map_err(|error| format!("Failed to resolve note {relative_path}: {error}"))?;
    if !target.starts_with(&root) {
        return Err("Refusing to read a note outside the active vault".to_owned());
    }
    let metadata = fs::metadata(&target).map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.len() > MAX_NOTE_BYTES {
        return Err("Addon note is not a regular file or exceeds the 5 MiB limit".to_owned());
    }
    let markdown = fs::read_to_string(&target)
        .map_err(|error| format!("Failed to read note {relative_path} as UTF-8: {error}"))?;
    Ok(json!({
        "path": relative_path,
        "size": metadata.len(),
        "modifiedAt": modified_at(&metadata),
        "markdown": markdown,
        "content": markdown
    }))
}

fn notes_write(root: &Path, addon: &InstalledAddon, params: &Value) -> Result<Value> {
    let relative_path = validate_markdown_path(params.get("path").and_then(Value::as_str).unwrap_or(""))?;
    let markdown = params
        .get("markdown")
        .or_else(|| params.get("content"))
        .and_then(Value::as_str)
        .unwrap_or("");
    if markdown.len() as u64 > MAX_NOTE_BYTES {
        return Err("Note exceeds the 5 MiB addon write limit".to_owned());
    }
    let scopes = note_write_scopes(addon);
    if !permitted(&scopes, &relative_path) {
        return Err(format!("Addon is not permitted to write {relative_path}"));
    }
    let root = canonical_root(root)?;
    let target = prepare_write_target(&root, &relative_path)?;
    let created = write_markdown_atomic(
        &target,
        markdown,
        params.get("overwrite").and_then(Value::as_bool).unwrap_or(false),
    )?;
    let metadata = fs::metadata(&target).map_err(|error| error.to_string())?;
    Ok(json!({
        "path": relative_path,
        "size": metadata.len(),
        "modifiedAt": modified_at(&metadata),
        "created": created
    }))
}

fn prepare_write_target(root: &Path, relative_path: &str) -> Result<PathBuf> {
    let relative = Path::new(relative_path);
    let file_name = relative
        .file_name()
        .ok_or_else(|| "A note file name is required".to_owned())?
        .to_os_string();
    let parent = relative.parent().unwrap_or_else(|| Path::new(""));
    let mut current = root.to_path_buf();
    for component in parent.components() {
        let Component::Normal(part) = component else {
            return Err("Unsafe addon note path".to_owned());
        };
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!("Addon note parent is a symbolic link: {}", current.display()));
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err(format!("Addon note parent is not a directory: {}", current.display()));
            }
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {
                fs::create_dir(&current).map_err(|error| error.to_string())?;
            }
            Err(error) => return Err(error.to_string()),
        }
        let canonical = fs::canonicalize(&current).map_err(|error| error.to_string())?;
        if !canonical.starts_with(root) {
            return Err("Refusing to write a note outside the active vault".to_owned());
        }
    }
    Ok(current.join(file_name))
}

fn write_markdown_atomic(target: &Path, markdown: &str, overwrite: bool) -> Result<bool> {
    let created = !target.exists();
    if !created {
        let metadata = fs::symlink_metadata(target).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err("Addon note target is not a regular file".to_owned());
        }
        if !overwrite {
            return Err("Addon note already exists and overwrite was not requested".to_owned());
        }
    }
    let file_name = target.file_name().and_then(|value| value.to_str()).unwrap_or("note.md");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    let temporary = target.with_file_name(format!(".{file_name}.{}-{nonce}.addonpart", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| error.to_string())?;
    file.write_all(markdown.as_bytes()).map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    drop(file);
    if !overwrite {
        match fs::hard_link(&temporary, target) {
            Ok(()) => {
                fs::remove_file(&temporary).map_err(|error| error.to_string())?;
                return Ok(true);
            }
            Err(error) => {
                let _ = fs::remove_file(&temporary);
                return Err(if matches!(error.kind(), ErrorKind::AlreadyExists | ErrorKind::PermissionDenied) {
                    "Addon note already exists and overwrite was not requested".to_owned()
                } else {
                    error.to_string()
                });
            }
        }
    }
    match fs::rename(&temporary, target) {
        Ok(()) => Ok(created),
        Err(error) if matches!(error.kind(), ErrorKind::AlreadyExists | ErrorKind::PermissionDenied) => {
            fs::remove_file(target).map_err(|error| error.to_string())?;
            fs::rename(&temporary, target).map_err(|error| error.to_string())?;
            Ok(false)
        }
        Err(error) => {
            let _ = fs::remove_file(&temporary);
            Err(error.to_string())
        }
    }
}

fn network_hosts(addon: &InstalledAddon) -> Vec<String> {
    permission_strings(addon, "/network/hosts")
}

fn host_matches(pattern: &str, host: &str) -> bool {
    let pattern = pattern.trim().to_ascii_lowercase();
    let host = host.to_ascii_lowercase();
    if pattern == "public-https" {
        true
    } else if let Some(suffix) = pattern.strip_prefix("*.") {
        host != suffix && host.ends_with(&format!(".{suffix}"))
    } else {
        host == pattern
    }
}

fn is_public_ipv4(address: Ipv4Addr) -> bool {
    let octets = address.octets();
    !address.is_private()
        && !address.is_loopback()
        && !address.is_link_local()
        && !address.is_broadcast()
        && !address.is_documentation()
        && !address.is_multicast()
        && !address.is_unspecified()
        && !(octets[0] == 100 && (64..=127).contains(&octets[1]))
        && !(octets[0] == 192 && octets[1] == 0 && octets[2] == 0)
        && !(octets[0] == 198 && (octets[1] == 18 || octets[1] == 19))
        && octets[0] != 0
        && octets[0] < 240
}

fn is_public_ipv6(address: Ipv6Addr) -> bool {
    if let Some(mapped) = address.to_ipv4_mapped() {
        return is_public_ipv4(mapped);
    }
    let segments = address.segments();
    let unique_local = segments[0] & 0xfe00 == 0xfc00;
    let link_local = segments[0] & 0xffc0 == 0xfe80;
    let site_local = segments[0] & 0xffc0 == 0xfec0;
    let documentation = segments[0] == 0x2001 && segments[1] == 0x0db8;
    let benchmarking = segments[0] == 0x2001 && segments[1] == 0x0002 && segments[2] == 0;
    let discard_only = segments[0] == 0x0100 && segments[1..4].iter().all(|segment| *segment == 0);
    !address.is_loopback()
        && !address.is_unspecified()
        && !address.is_multicast()
        && !unique_local
        && !link_local
        && !site_local
        && !documentation
        && !benchmarking
        && !discard_only
}

fn is_public_ip(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(value) => is_public_ipv4(value),
        IpAddr::V6(value) => is_public_ipv6(value),
    }
}

fn validate_url(url: &Url, allowed_hosts: &[String]) -> Result<(String, SocketAddr)> {
    if url.scheme() != "https" {
        return Err("External addons may only request HTTPS URLs".to_owned());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("Credentials in addon URLs are not allowed".to_owned());
    }
    let host = url.host_str().ok_or_else(|| "URL has no host".to_owned())?.to_ascii_lowercase();
    if !allowed_hosts.iter().any(|pattern| host_matches(pattern, &host)) {
        return Err(format!("Network access to {host} was not granted"));
    }
    let port = url.port_or_known_default().ok_or_else(|| "URL has no HTTPS port".to_owned())?;
    if port != 443 {
        return Err("External addon HTTPS requests are restricted to port 443".to_owned());
    }
    let addresses = (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|error| format!("Failed to resolve {host}: {error}"))?
        .collect::<Vec<_>>();
    if addresses.is_empty() || addresses.iter().any(|address| !is_public_ip(address.ip())) {
        return Err(format!("Network access to a local/private address for {host} is blocked"));
    }
    Ok((host, addresses[0]))
}

fn parse_method(params: &Value) -> Result<Method> {
    let name = params
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or("GET")
        .to_ascii_uppercase();
    let method = Method::from_bytes(name.as_bytes()).map_err(|error| error.to_string())?;
    if !matches!(method, Method::GET | Method::POST | Method::PUT | Method::PATCH | Method::DELETE) {
        return Err(format!("Unsupported HTTP method: {name}"));
    }
    Ok(method)
}

fn client_for(host: &str, address: SocketAddr) -> Result<Client> {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .resolve(host, address)
        .build()
        .map_err(|error| error.to_string())
}

fn send_request(client: &Client, method: Method, url: Url, params: &Value) -> Result<Response> {
    let mut request = client.request(method, url);
    if let Some(headers) = params.get("headers").and_then(Value::as_object) {
        for (name, value) in headers {
            if matches!(name.to_ascii_lowercase().as_str(), "host" | "content-length" | "cookie" | "proxy-authorization") {
                continue;
            }
            if let Some(value) = value.as_str() {
                request = request.header(name, value);
            }
        }
    }
    if let Some(body) = params.get("body").and_then(Value::as_str) {
        if body.len() > MAX_HTTP_REQUEST_BYTES {
            return Err("HTTP request body exceeds the 1 MiB limit".to_owned());
        }
        request = request.body(body.to_owned());
    }
    request.send().map_err(|error| error.to_string())
}

fn read_response(mut response: Response) -> Result<Value> {
    if response.content_length().is_some_and(|length| length > MAX_HTTP_RESPONSE_BYTES) {
        return Err("HTTP response exceeds the 5 MiB limit".to_owned());
    }
    let status = response.status().as_u16();
    let headers = response
        .headers()
        .iter()
        .filter_map(|(name, value)| value.to_str().ok().map(|value| (name.to_string(), Value::String(value.to_owned()))))
        .collect::<Map<String, Value>>();
    let mut bytes = Vec::new();
    response
        .by_ref()
        .take(MAX_HTTP_RESPONSE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > MAX_HTTP_RESPONSE_BYTES {
        return Err("HTTP response exceeds the 5 MiB limit".to_owned());
    }
    Ok(json!({
        "status": status,
        "ok": (200..300).contains(&status),
        "headers": headers,
        "body": String::from_utf8_lossy(&bytes).to_string()
    }))
}

fn http_request(addon: &InstalledAddon, params: &Value) -> Result<Value> {
    let hosts = network_hosts(addon);
    let raw_url = params.get("url").and_then(Value::as_str).unwrap_or("");
    let mut url = Url::parse(raw_url).map_err(|error| format!("Invalid URL: {error}"))?;
    url.set_fragment(None);
    let method = parse_method(params)?;
    for redirect_count in 0..=MAX_REDIRECTS {
        let (host, address) = validate_url(&url, &hosts)?;
        let response = send_request(&client_for(&host, address)?, method.clone(), url.clone(), params)?;
        if !response.status().is_redirection() {
            return read_response(response);
        }
        if method != Method::GET {
            return Err("Redirects are only allowed for GET requests".to_owned());
        }
        if redirect_count == MAX_REDIRECTS {
            return Err(format!("HTTP request exceeded {MAX_REDIRECTS} redirects"));
        }
        let location = response
            .headers()
            .get(LOCATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| "Redirect response did not include a valid Location header".to_owned())?;
        url = url.join(location).map_err(|error| format!("Invalid redirect URL: {error}"))?;
        url.set_fragment(None);
    }
    Err("HTTP redirect handling failed".to_owned())
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or_else(|| "Atomic write has no parent directory".to_owned())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&temporary)
        .map_err(|error| error.to_string())?;
    file.write_all(bytes).map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    drop(file);
    #[cfg(target_os = "windows")]
    if path.exists() {
        fs::remove_file(path).map_err(|error| error.to_string())?;
    }
    fs::rename(&temporary, path).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addon_adapter::{AddonManifest, AddonRuntime};

    fn addon(permissions: Value) -> InstalledAddon {
        let mut extra = Map::new();
        extra.insert("permissions".to_owned(), permissions);
        InstalledAddon {
            manifest: AddonManifest {
                id: "fixture.external".to_owned(),
                name: "Fixture".to_owned(),
                version: "1.0.0".to_owned(),
                api_version: 1,
                runtime: AddonRuntime { kind: "javascript-worker".to_owned(), entry: "main.js".to_owned() },
                extra,
                ..AddonManifest::default()
            },
            enabled: true,
            source: "external".to_owned(),
            ..InstalledAddon::default()
        }
    }

    fn temp_root(label: &str) -> PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("elephant-addon-host-{label}-{stamp}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn storage_is_private_and_persistent() {
        let root = temp_root("storage");
        let addon = addon(json!({}));
        assert_eq!(handle(&root, &addon, "storage.set", &json!({"key":"answer","value":42})).unwrap(), true);
        assert_eq!(handle(&root, &addon, "storage.get", &json!({"key":"answer"})).unwrap(), 42);
        let entries = handle(&root, &addon, "storage.entries", &json!({})).unwrap();
        assert_eq!(entries["answer"], 42);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn note_permissions_are_deny_by_default_and_boundary_aware() {
        let root = temp_root("notes");
        fs::create_dir_all(root.join("Inbox")).unwrap();
        fs::write(root.join("Inbox/note.md"), "# Allowed").unwrap();
        fs::write(root.join("outside.md"), "# Denied").unwrap();
        let denied = addon(json!({}));
        assert!(handle(&root, &denied, "notes.read", &json!({"path":"Inbox/note.md"})).is_err());
        let allowed = addon(json!({"notes":{"read":["Inbox/**"],"write":["Inbox/**"]}}));
        assert_eq!(handle(&root, &allowed, "notes.read", &json!({"path":"Inbox/note.md"})).unwrap()["markdown"], "# Allowed");
        assert!(handle(&root, &allowed, "notes.read", &json!({"path":"outside.md"})).is_err());
        assert!(handle(&root, &allowed, "notes.read", &json!({"path":"../outside.md"})).is_err());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn network_rejects_private_and_ungranted_targets_before_request() {
        let denied = addon(json!({"network":{"hosts":[]}}));
        assert!(http_request(&denied, &json!({"url":"https://127.0.0.1/"})).is_err());
        assert!(!is_public_ip("127.0.0.1".parse().unwrap()));
        assert!(!is_public_ip("10.0.0.1".parse().unwrap()));
        assert!(is_public_ip("1.1.1.1".parse().unwrap()));
    }
}