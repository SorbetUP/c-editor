use reqwest::{
  blocking::{Client, Response},
  header::LOCATION,
  Method,
};
use serde_json::{json, Map, Value};
use std::{
  io::Read,
  net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs},
  time::Duration,
};
use tauri::AppHandle;
use url::Url;

use crate::addon_runtime_access::{host_matches, read_enabled_addon};

type R<T> = Result<T, String>;

const MAX_HTTP_REQUEST_BYTES: usize = 1024 * 1024;
const MAX_HTTP_RESPONSE_BYTES: u64 = 5 * 1024 * 1024;
const MAX_REDIRECTS: usize = 5;

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
    IpAddr::V4(address) => is_public_ipv4(address),
    IpAddr::V6(address) => is_public_ipv6(address),
  }
}

fn validate_url(url: &Url, allowed_hosts: &[String]) -> R<(String, SocketAddr)> {
  if url.scheme() != "https" {
    return Err("External addons may only request HTTPS URLs".to_string());
  }
  if !url.username().is_empty() || url.password().is_some() {
    return Err("Credentials in addon URLs are not allowed".to_string());
  }
  let host = url.host_str().ok_or_else(|| "URL has no host".to_string())?.to_ascii_lowercase();
  if !allowed_hosts.iter().any(|pattern| host_matches(pattern, &host)) {
    return Err(format!("Network access to {host} was not granted"));
  }
  let port = url.port_or_known_default().ok_or_else(|| "URL has no HTTPS port".to_string())?;
  if port != 443 {
    return Err("External addon HTTPS requests are restricted to port 443".to_string());
  }

  let addresses = (host.as_str(), port)
    .to_socket_addrs()
    .map_err(|error| format!("Failed to resolve {host}: {error}"))?
    .collect::<Vec<_>>();
  if addresses.is_empty() {
    return Err(format!("No address resolved for {host}"));
  }
  if addresses.iter().any(|address| !is_public_ip(address.ip())) {
    return Err(format!("Network access to a local or private address for {host} is blocked"));
  }
  Ok((host, addresses[0]))
}

fn client_for(host: &str, address: SocketAddr) -> R<Client> {
  Client::builder()
    .timeout(Duration::from_secs(30))
    .redirect(reqwest::redirect::Policy::none())
    .resolve(host, address)
    .build()
    .map_err(|error| error.to_string())
}

fn parse_method(params: &Value) -> R<Method> {
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

fn send_request(client: &Client, method: Method, url: Url, params: &Value) -> R<Response> {
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
      return Err("HTTP request body exceeds the 1 MiB limit".to_string());
    }
    request = request.body(body.to_string());
  }
  request.send().map_err(|error| error.to_string())
}

fn read_response(mut response: Response) -> R<Value> {
  if response.content_length().is_some_and(|length| length > MAX_HTTP_RESPONSE_BYTES) {
    return Err("HTTP response exceeds the 5 MiB limit".to_string());
  }
  let status = response.status().as_u16();
  let headers = response
    .headers()
    .iter()
    .filter_map(|(name, value)| value.to_str().ok().map(|value| (name.to_string(), Value::String(value.to_string()))))
    .collect::<Map<String, Value>>();
  let mut bytes = Vec::new();
  response
    .by_ref()
    .take(MAX_HTTP_RESPONSE_BYTES + 1)
    .read_to_end(&mut bytes)
    .map_err(|error| error.to_string())?;
  if bytes.len() as u64 > MAX_HTTP_RESPONSE_BYTES {
    return Err("HTTP response exceeds the 5 MiB limit".to_string());
  }
  Ok(json!({
    "status": status,
    "ok": (200..300).contains(&status),
    "headers": headers,
    "body": String::from_utf8_lossy(&bytes).to_string()
  }))
}

#[tauri::command]
pub fn tauri_addons_http_request(
  app: AppHandle,
  addon_id: String,
  params: Option<Value>,
) -> R<Value> {
  let record = read_enabled_addon(&app, &addon_id)?;
  let params = params.unwrap_or_else(|| Value::Object(Map::new()));
  let raw_url = params.get("url").and_then(Value::as_str).unwrap_or("");
  let mut url = Url::parse(raw_url).map_err(|error| format!("Invalid URL: {error}"))?;
  url.set_fragment(None);
  let method = parse_method(&params)?;

  for redirect_count in 0..=MAX_REDIRECTS {
    let (host, address) = validate_url(&url, &record.manifest.permissions.network.hosts)?;
    let response = send_request(&client_for(&host, address)?, method.clone(), url.clone(), &params)?;
    if !response.status().is_redirection() {
      return read_response(response);
    }
    if method != Method::GET {
      return Err("Redirects are only allowed for GET requests".to_string());
    }
    if redirect_count == MAX_REDIRECTS {
      return Err(format!("HTTP request exceeded {MAX_REDIRECTS} redirects"));
    }
    let location = response
      .headers()
      .get(LOCATION)
      .and_then(|value| value.to_str().ok())
      .ok_or_else(|| "Redirect response did not include a valid Location header".to_string())?;
    url = url.join(location).map_err(|error| format!("Invalid redirect URL: {error}"))?;
    url.set_fragment(None);
  }

  Err("HTTP redirect handling failed".to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn blocks_private_and_special_addresses() {
    assert!(!is_public_ip("127.0.0.1".parse().unwrap()));
    assert!(!is_public_ip("10.0.0.1".parse().unwrap()));
    assert!(!is_public_ip("100.64.0.1".parse().unwrap()));
    assert!(!is_public_ip("169.254.1.1".parse().unwrap()));
    assert!(!is_public_ip("::1".parse().unwrap()));
    assert!(!is_public_ip("fc00::1".parse().unwrap()));
    assert!(!is_public_ip("::ffff:127.0.0.1".parse().unwrap()));
    assert!(!is_public_ip("::ffff:10.0.0.1".parse().unwrap()));
    assert!(!is_public_ip("2001:db8::1".parse().unwrap()));
    assert!(!is_public_ip("2001:2::1".parse().unwrap()));
    assert!(!is_public_ip("100::1".parse().unwrap()));
    assert!(is_public_ip("1.1.1.1".parse().unwrap()));
    assert!(is_public_ip("2606:4700:4700::1111".parse().unwrap()));
  }

  #[test]
  fn rejects_credentials_and_non_https_urls() {
    let hosts = vec!["example.com".to_string()];
    assert!(validate_url(&Url::parse("http://example.com").unwrap(), &hosts).is_err());
    assert!(validate_url(&Url::parse("https://user:pass@example.com").unwrap(), &hosts).is_err());
  }
}
