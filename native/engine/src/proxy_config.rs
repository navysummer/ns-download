use std::collections::HashMap;
use crate::downloader::DownloadError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyMode {
    None,
    System,
    Manual,
}

impl ProxyMode {
    pub fn parse_str(s: &str) -> Self {
        match s {
            "system" => Self::System,
            "manual" => Self::Manual,
            _ => Self::None,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::System => "system",
            Self::Manual => "manual",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyType {
    Http,
    Https,
    Socks4,
    Socks5,
}

impl ProxyType {
    pub fn parse_str(s: &str) -> Self {
        match s {
            "https" => Self::Https,
            "socks4" => Self::Socks4,
            "socks5" => Self::Socks5,
            _ => Self::Http,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
            Self::Socks4 => "socks4",
            Self::Socks5 => "socks5",
        }
    }
    pub fn scheme(&self) -> &'static str {
        self.as_str()
    }
    pub fn is_socks(&self) -> bool {
        matches!(self, Self::Socks4 | Self::Socks5)
    }
}

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub mode: ProxyMode,
    pub proxy_type: ProxyType,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub no_proxy_list: String,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            mode: ProxyMode::None,
            proxy_type: ProxyType::Http,
            host: String::new(),
            port: 0,
            username: String::new(),
            password: String::new(),
            no_proxy_list: String::new(),
        }
    }
}

impl ProxyConfig {
    pub fn from_config_map(map: &HashMap<String, String>) -> Self {
        let mode = map.get("proxy_mode").map(|v| ProxyMode::parse_str(v)).unwrap_or(ProxyMode::None);
        let proxy_type = map.get("proxy_type").map(|v| ProxyType::parse_str(v)).unwrap_or(ProxyType::Http);
        let host = map.get("proxy_host").cloned().unwrap_or_default();
        let port = map.get("proxy_port").and_then(|v| v.parse::<u16>().ok()).unwrap_or(0);
        let username = map.get("proxy_username").cloned().unwrap_or_default();
        let password = map.get("proxy_password").cloned().unwrap_or_default();
        let no_proxy_list = map.get("proxy_no_list").cloned().unwrap_or_default();
        Self { mode, proxy_type, host, port, username, password, no_proxy_list }
    }

    pub fn is_active(&self) -> bool { self.mode != ProxyMode::None }
    pub fn is_socks(&self) -> bool { self.proxy_type.is_socks() }

    pub fn to_proxy_url(&self) -> Option<String> {
        match self.mode {
            ProxyMode::None => None,
            ProxyMode::System => None,
            ProxyMode::Manual => {
                if self.host.is_empty() || self.port == 0 { return None; }
                let scheme = self.proxy_type.scheme();
                if !self.username.is_empty() {
                    let enc_user = percent_encode_userinfo(&self.username);
                    let enc_pass = percent_encode_userinfo(&self.password);
                    Some(format!("{}://{}:{}@{}:{}", scheme, enc_user, enc_pass, self.host, self.port))
                } else {
                    Some(format!("{}://{}:{}", scheme, self.host, self.port))
                }
            }
        }
    }

    pub fn resolve(&self) -> Self {
        match self.mode {
            ProxyMode::System => detect_system_proxy().ok().flatten().unwrap_or_default(),
            _ => self.clone(),
        }
    }

    pub fn addr(&self) -> String { format!("{}:{}", self.host, self.port) }

    pub fn from_proxy_url(url: &str) -> Self {
        if url.is_empty() { return Self::default(); }
        let (scheme, rest) = if let Some(idx) = url.find("://") { (&url[..idx], &url[idx + 3..]) } else { ("http", url) };
        let proxy_type = ProxyType::parse_str(scheme);
        let (auth, host_port) = if let Some(at_idx) = rest.rfind('@') { (&rest[..at_idx], &rest[at_idx + 1..]) } else { ("", rest) };
        let (username, password) = if auth.is_empty() {
            (String::new(), String::new())
        } else if let Some(colon) = auth.find(':') {
            (percent_decode(&auth[..colon]), percent_decode(&auth[colon + 1..]))
        } else {
            (percent_decode(auth), String::new())
        };
        let (host, port) = parse_host_port(host_port);
        Self { mode: ProxyMode::Manual, proxy_type, host, port, username, password, no_proxy_list: String::new() }
    }
}

#[cfg(target_os = "windows")]
pub fn detect_system_proxy() -> Result<Option<ProxyConfig>, DownloadError> {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let inet = hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Internet Settings")
        .map_err(|e| DownloadError::Other(format!("failed to open Internet Settings: {}", e)))?;
    let enabled: u32 = inet.get_value("ProxyEnable").unwrap_or(0);
    if enabled == 0 { return Ok(None); }
    let server: String = inet.get_value("ProxyServer").unwrap_or_default();
    if server.is_empty() { return Ok(None); }
    let bypass: String = inet.get_value("ProxyOverride").unwrap_or_default();
    let no_proxy = bypass.replace(';', ",").replace("<local>", "localhost");
    let (proxy_type, host, port) = parse_windows_proxy_server(&server);
    Ok(Some(ProxyConfig { mode: ProxyMode::Manual, proxy_type, host, port, username: String::new(), password: String::new(), no_proxy_list: no_proxy }))
}

#[cfg(not(target_os = "windows"))]
pub fn detect_system_proxy() -> Result<Option<ProxyConfig>, DownloadError> { Ok(None) }

pub fn parse_windows_proxy_server(server: &str) -> (ProxyType, String, u16) {
    if server.contains('=') {
        let entries = parse_multi_protocol_proxy(server);
        if let Some((host, port)) = entries.get("socks") { return (ProxyType::Socks5, host.clone(), *port); }
        if let Some((host, port)) = entries.get("https") { return (ProxyType::Https, host.clone(), *port); }
        if let Some((host, port)) = entries.get("http") { return (ProxyType::Http, host.clone(), *port); }
        if let Some((_key, (host, port))) = entries.into_iter().next() { return (ProxyType::Http, host, port); }
    }
    let (host, port) = parse_host_port(server);
    (ProxyType::Http, host, port)
}

fn parse_multi_protocol_proxy(server: &str) -> HashMap<String, (String, u16)> {
    let mut result = HashMap::new();
    for entry in server.split(';') {
        let entry = entry.trim();
        if entry.is_empty() { continue; }
        if let Some((protocol, addr)) = entry.split_once('=') {
            let protocol = protocol.trim().to_ascii_lowercase();
            let (host, port) = parse_host_port(addr.trim());
            if !host.is_empty() { result.insert(protocol, (host, port)); }
        }
    }
    result
}

fn parse_host_port(addr: &str) -> (String, u16) {
    if let Some(bracket_end) = addr.find(']') {
        let host = addr[..=bracket_end].to_string();
        let rest = &addr[bracket_end + 1..];
        let port = rest.strip_prefix(':').and_then(|p| p.parse::<u16>().ok()).unwrap_or(8080);
        return (host, port);
    }
    if let Some(colon) = addr.rfind(':') {
        let host = addr[..colon].to_string();
        let port = addr[colon + 1..].parse::<u16>().unwrap_or(8080);
        if !host.is_empty() { return (host, port); }
    }
    if !addr.is_empty() { return (addr.to_string(), 8080); }
    (String::new(), 0)
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn percent_decode(s: &str) -> String {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len()
            && let (Some(hi), Some(lo)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2]))
        {
            result.push((hi << 4) | lo);
            i += 3;
            continue;
        }
        result.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(result).unwrap_or_else(|_| s.to_string())
}

fn percent_encode_userinfo(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => result.push(b as char),
            b'!' | b'$' | b'&' | b'\'' | b'(' | b')' | b'*' | b'+' | b',' | b';' | b'=' => result.push(b as char),
            _ => { result.push('%'); result.push_str(&format!("{:02X}", b)); }
        }
    }
    result
}

pub fn socks5_connect_sync(
    proxy_host: &str, proxy_port: u16, target_host: &str, target_port: u16,
    username: &str, password: &str, timeout: std::time::Duration,
) -> Result<std::net::TcpStream, DownloadError> {
    use std::net::TcpStream;
    let proxy_addr = format!("{}:{}", proxy_host, proxy_port);
    let sock_addr: std::net::SocketAddr = proxy_addr.parse().or_else(|_| {
        use std::net::ToSocketAddrs;
        proxy_addr.to_socket_addrs()
            .map_err(|e| DownloadError::Other(format!("proxy DNS resolve error: {}", e)))?
            .next().ok_or_else(|| DownloadError::Other("proxy DNS returned no addresses".to_string()))
    })?;
    let stream = TcpStream::connect_timeout(&sock_addr, timeout)
        .map_err(|e| DownloadError::Other(format!("SOCKS5 proxy connect error: {}", e)))?;
    stream.set_read_timeout(Some(timeout)).ok();
    stream.set_write_timeout(Some(timeout)).ok();
    socks5_handshake(stream, target_host, target_port, username, password)
}

fn socks5_handshake(
    mut stream: std::net::TcpStream, target_host: &str, target_port: u16,
    username: &str, password: &str,
) -> Result<std::net::TcpStream, DownloadError> {
    use std::io::{Read, Write};
    let need_auth = !username.is_empty();
    let greeting = if need_auth { vec![0x05, 0x02, 0x00, 0x02] } else { vec![0x05, 0x01, 0x00] };
    stream.write_all(&greeting).map_err(|e| DownloadError::Other(format!("SOCKS5 greeting write error: {}", e)))?;
    let mut method_resp = [0u8; 2];
    stream.read_exact(&mut method_resp).map_err(|e| DownloadError::Other(format!("SOCKS5 method response read error: {}", e)))?;
    if method_resp[0] != 0x05 { return Err(DownloadError::Other(format!("SOCKS5 protocol error: unexpected version {}", method_resp[0]))); }
    match method_resp[1] {
        0x00 => {}
        0x02 => { socks5_auth(&mut stream, username, password)?; }
        0xFF => { return Err(DownloadError::Other("SOCKS5 proxy rejected all authentication methods".to_string())); }
        other => { return Err(DownloadError::Other(format!("SOCKS5 unsupported auth method: 0x{:02x}", other))); }
    }
    if target_host.len() > 255 { return Err(DownloadError::Other(format!("SOCKS5 target hostname too long: {} bytes (max 255)", target_host.len()))); }
    let mut connect_req = vec![0x05, 0x01, 0x00, 0x03, target_host.len() as u8];
    connect_req.extend_from_slice(target_host.as_bytes());
    connect_req.push((target_port >> 8) as u8);
    connect_req.push((target_port & 0xFF) as u8);
    stream.write_all(&connect_req).map_err(|e| DownloadError::Other(format!("SOCKS5 connect write error: {}", e)))?;
    let mut resp = [0u8; 4];
    stream.read_exact(&mut resp).map_err(|e| DownloadError::Other(format!("SOCKS5 connect response read error: {}", e)))?;
    if resp[0] != 0x05 || resp[1] != 0x00 { return Err(DownloadError::Other(format!("SOCKS5 connect failed: status 0x{:02x}", resp[1]))); }
    let _ = Vec::<u8>::new();
    match resp[3] {
        0x01 => { let mut buf = [0u8; 4]; stream.read_exact(&mut buf)?; }
        0x03 => { let mut len = [0u8; 1]; stream.read_exact(&mut len)?; let mut buf = vec![0u8; len[0] as usize]; stream.read_exact(&mut buf)?; }
        0x04 => { let mut buf = [0u8; 16]; stream.read_exact(&mut buf)?; }
        _ => {}
    }
    let mut port_buf = [0u8; 2];
    stream.read_exact(&mut port_buf)?;
    stream.set_read_timeout(None).ok();
    stream.set_write_timeout(None).ok();
    Ok(stream)
}

fn socks5_auth(stream: &mut std::net::TcpStream, username: &str, password: &str) -> Result<(), DownloadError> {
    use std::io::{Read, Write};
    if username.len() > 255 || password.len() > 255 {
        return Err(DownloadError::Other("SOCKS5 username/password too long".to_string()));
    }
    let mut req = vec![0x01, username.len() as u8];
    req.extend_from_slice(username.as_bytes());
    req.push(password.len() as u8);
    req.extend_from_slice(password.as_bytes());
    stream.write_all(&req).map_err(|e| DownloadError::Other(format!("SOCKS5 auth write error: {}", e)))?;
    let mut resp = [0u8; 2];
    stream.read_exact(&mut resp).map_err(|e| DownloadError::Other(format!("SOCKS5 auth response read error: {}", e)))?;
    if resp[1] != 0x00 { return Err(DownloadError::Other("SOCKS5 authentication failed".to_string())); }
    Ok(())
}

pub fn socks4_connect_sync(
    proxy_host: &str, proxy_port: u16, target_host: &str, target_port: u16,
    timeout: std::time::Duration,
) -> Result<std::net::TcpStream, DownloadError> {
    use std::io::{Read, Write};
    use std::net::{TcpStream, ToSocketAddrs};
    let proxy_addr = format!("{}:{}", proxy_host, proxy_port);
    let sock_addr: std::net::SocketAddr = proxy_addr.parse().or_else(|_| {
        proxy_addr.to_socket_addrs()
            .map_err(|e| DownloadError::Other(format!("proxy DNS resolve error: {}", e)))?
            .next().ok_or_else(|| DownloadError::Other("proxy DNS returned no addresses".to_string()))
    })?;
    let mut stream = TcpStream::connect_timeout(&sock_addr, timeout)
        .map_err(|e| DownloadError::Other(format!("SOCKS4 proxy connect error: {}", e)))?;
    stream.set_read_timeout(Some(timeout)).ok();
    stream.set_write_timeout(Some(timeout)).ok();
    let target_addr = format!("{}:{}", target_host, target_port);
    let target_ip = target_addr.to_socket_addrs()
        .map_err(|e| DownloadError::Other(format!("target DNS resolve error: {}", e)))?
        .find(|a| a.is_ipv4())
        .ok_or_else(|| DownloadError::Other("SOCKS4 requires IPv4 address".to_string()))?;
    let ip_bytes = match target_ip.ip() {
        std::net::IpAddr::V4(ipv4) => ipv4.octets(),
        _ => return Err(DownloadError::Other("SOCKS4 requires IPv4 address".to_string())),
    };
    let req = vec![0x04, 0x01, (target_port >> 8) as u8, (target_port & 0xFF) as u8, ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3], 0x00];
    stream.write_all(&req).map_err(|e| DownloadError::Other(format!("SOCKS4 request write error: {}", e)))?;
    let mut resp = [0u8; 8];
    stream.read_exact(&mut resp).map_err(|e| DownloadError::Other(format!("SOCKS4 response read error: {}", e)))?;
    if resp[1] != 0x5A { return Err(DownloadError::Other(format!("SOCKS4 connect failed: status 0x{:02x}", resp[1]))); }
    stream.set_read_timeout(None).ok();
    stream.set_write_timeout(None).ok();
    Ok(stream)
}

pub fn socks_connect_sync(
    proxy: &ProxyConfig, target_host: &str, target_port: u16, timeout: std::time::Duration,
) -> Result<std::net::TcpStream, DownloadError> {
    match proxy.proxy_type {
        ProxyType::Socks5 => socks5_connect_sync(&proxy.host, proxy.port, target_host, target_port, &proxy.username, &proxy.password, timeout),
        ProxyType::Socks4 => socks4_connect_sync(&proxy.host, proxy.port, target_host, target_port, timeout),
        _ => Err(DownloadError::Other(format!("socks_connect_sync called with non-SOCKS proxy type: {}", proxy.proxy_type.as_str()))),
    }
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 { result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char); } else { result.push('='); }
        if chunk.len() > 2 { result.push(CHARS[(triple & 0x3F) as usize] as char); } else { result.push('='); }
    }
    result
}

fn parse_connect_status_line(header: &[u8]) -> (u16, String) {
    let status_line: &[u8] = header.split(|&b| b == b'\n').next().unwrap_or(header);
    let status_line_str = String::from_utf8_lossy(status_line).into_owned();
    let status_code = status_line_str.split_whitespace().nth(1).and_then(|s| s.parse::<u16>().ok()).unwrap_or(0);
    (status_code, status_line_str)
}

pub fn http_connect_proxy_sync(
    proxy: &ProxyConfig, target_host: &str, target_port: u16, timeout: std::time::Duration,
) -> Result<std::net::TcpStream, DownloadError> {
    use std::io::{Read, Write};
    use std::net::{TcpStream, ToSocketAddrs};
    let proxy_addr = format!("{}:{}", proxy.host, proxy.port);
    let sock_addr: std::net::SocketAddr = proxy_addr.parse().or_else(|_| {
        proxy_addr.to_socket_addrs()
            .map_err(|e| DownloadError::Other(format!("proxy DNS resolve error: {}", e)))?
            .next().ok_or_else(|| DownloadError::Other("proxy DNS returned no addresses".to_string()))
    })?;
    let mut stream = TcpStream::connect_timeout(&sock_addr, timeout)
        .map_err(|e| DownloadError::Other(format!("HTTP CONNECT proxy connect error: {}", e)))?;
    stream.set_read_timeout(Some(timeout)).ok();
    stream.set_write_timeout(Some(timeout)).ok();
    let target = format!("{}:{}", target_host, target_port);
    let mut req = format!("CONNECT {} HTTP/1.1\r\nHost: {}\r\n", target, target);
    if !proxy.username.is_empty() {
        use std::fmt::Write as FmtWrite;
        let credentials = format!("{}:{}", proxy.username, proxy.password);
        let encoded = base64_encode(credentials.as_bytes());
        let _ = write!(req, "Proxy-Authorization: Basic {}\r\n", encoded);
    }
    req.push_str("\r\n");
    stream.write_all(req.as_bytes()).map_err(|e| DownloadError::Other(format!("HTTP CONNECT write error: {}", e)))?;
    const MAX_HEADER_BYTES: usize = 64 * 1024;
    let mut header = Vec::with_capacity(256);
    let mut byte = [0u8; 1];
    loop {
        let n = stream.read(&mut byte).map_err(|e| DownloadError::Other(format!("HTTP CONNECT read error: {}", e)))?;
        if n == 0 { return Err(DownloadError::Other("HTTP CONNECT proxy closed connection before sending full response".to_string())); }
        header.push(byte[0]);
        if header.ends_with(b"\r\n\r\n") { break; }
        if header.len() >= MAX_HEADER_BYTES { return Err(DownloadError::Other("HTTP CONNECT response header exceeded maximum size".to_string())); }
    }
    let (status_code, status_line) = parse_connect_status_line(&header);
    if status_code != 200 { return Err(DownloadError::Other(format!("HTTP CONNECT failed: {}", status_line.trim()))); }
    stream.set_read_timeout(None).ok();
    stream.set_write_timeout(None).ok();
    Ok(stream)
}

pub fn proxy_connect_sync(
    proxy: &ProxyConfig, target_host: &str, target_port: u16, timeout: std::time::Duration,
) -> Result<std::net::TcpStream, DownloadError> {
    match proxy.proxy_type {
        ProxyType::Socks4 | ProxyType::Socks5 => socks_connect_sync(proxy, target_host, target_port, timeout),
        ProxyType::Http | ProxyType::Https => http_connect_proxy_sync(proxy, target_host, target_port, timeout),
    }
}

const CONNECTIVITY_CHECK_URLS: &[&str] = &[
    "http://www.msftconnecttest.com/connecttest.txt",
    "http://cp.cloudflare.com",
    "http://connectivitycheck.gstatic.com/generate_204",
];

pub async fn test_proxy_connection(
    proxy_type: &str, proxy_host: &str, proxy_port: &str,
    proxy_username: &str, proxy_password: &str,
) -> Result<i64, DownloadError> {
    use std::time::Instant;
    let config = ProxyConfig {
        mode: ProxyMode::Manual,
        proxy_type: ProxyType::parse_str(proxy_type),
        host: proxy_host.to_string(),
        port: proxy_port.parse::<u16>().unwrap_or(0),
        username: proxy_username.to_string(),
        password: proxy_password.to_string(),
        no_proxy_list: String::new(),
    };
    let proxy_url = config.to_proxy_url().ok_or_else(|| DownloadError::Other("incomplete proxy config".to_string()))?;
    log_info!("[proxy-test] testing proxy: {}", proxy_url);
    let mut proxy = reqwest::Proxy::all(&proxy_url).map_err(|e| DownloadError::Other(format!("invalid proxy URL: {}", e)))?;
    if !proxy_username.is_empty() { proxy = proxy.basic_auth(proxy_username, proxy_password); }
    let client = reqwest::Client::builder()
        .proxy(proxy)
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(15))
        .build().map_err(|e| DownloadError::Other(format!("failed to build test client: {}", e)))?;
    let mut last_err = String::new();
    for url in CONNECTIVITY_CHECK_URLS {
        log_info!("[proxy-test] trying: {}", url);
        let start = Instant::now();
        match client.head(*url).send().await {
            Ok(resp) => {
                let latency = start.elapsed().as_millis() as i64;
                let status = resp.status();
                log_info!("[proxy-test] {} -> status={}, latency={}ms", url, status, latency);
                if !status.is_server_error() { return Ok(latency); }
                last_err = format!("{}: HTTP {}", url, status);
            }
            Err(e) => { log_info!("[proxy-test] {} -> error: {}", url, e); last_err = format!("{}: {}", url, e); }
        }
    }
    Err(DownloadError::Other(format!("all connectivity checks failed, last: {}", last_err)))
}
