use std::collections::HashMap;
use ns_download_engine::Engine;
use ns_download_engine::proxy_config::{ProxyConfig, ProxyMode, ProxyType};
use ns_download_engine::bt_downloader::BtConfig;

fn str_to_bool(s: &str, default: bool) -> bool {
    match s {
        "true" => true,
        "false" => false,
        _ => default,
    }
}

fn str_to_u64(s: &str, default: u64) -> u64 {
    s.parse::<u64>().unwrap_or(default)
}

fn str_to_i32(s: &str, default: i32) -> i32 {
    s.parse::<i32>().unwrap_or(default)
}

fn str_to_usize(s: &str, default: usize) -> usize {
    s.parse::<usize>().unwrap_or(default)
}

/// Load all settings from the engine DB as a HashMap.
pub async fn load_all(engine: &Engine) -> Result<HashMap<String, String>, String> {
    engine.db.get_all_config().await.map_err(|e| e.to_string())
}

/// Save multiple settings at once.
pub async fn save_all_and_apply(
    engine: &mut Engine,
    settings: &HashMap<String, String>,
) -> Result<(), String> {
    for (key, value) in settings {
        engine.db.set_config(key, value).await.map_err(|e| e.to_string())?;
    }
    for (key, value) in settings {
        apply_config(engine, key, value).await;
    }
    Ok(())
}

/// Apply a single config key to the running engine.
async fn apply_config(engine: &mut Engine, key: &str, value: &str) {
    match key {
        "max_concurrent_tasks" => {
            engine.manager.set_max_concurrent(str_to_usize(value, 5)).await;
        }
        "speed_limit_bytes" => {
            engine.manager.set_speed_limit(str_to_u64(value, 0));
        }
        "max_auto_retries" => {
            engine.manager.set_max_auto_retries(str_to_i32(value, 3));
        }
        "auto_retry_delay_secs" => {
            engine.manager.set_auto_retry_delay_secs(str_to_u64(value, 5));
        }
        "default_save_dir" => {
            engine.manager.set_default_save_dir(value.to_string());
        }
        "default_segments" => {
            engine.manager.set_default_segments(str_to_i32(value, 0));
        }
        "auto_max_connections" => {
            engine.manager.set_auto_max_connections(str_to_i32(value, 16));
        }
        "use_server_time" => {
            engine.manager.set_use_server_time(str_to_bool(value, false));
        }
        "global_user_agent" => {
            let _ = engine.manager.set_user_agent(value.to_string());
        }
        "proxy_mode" | "proxy_type" | "proxy_host" | "proxy_port"
        | "proxy_username" | "proxy_password" | "proxy_no_list" => {
            apply_proxy_config(engine, key, value).await;
        }
        "bt_enable_dht" | "bt_enable_upnp" | "bt_port_start" | "bt_port_end"
        | "bt_custom_trackers" | "bt_tracker_sub_urls" | "bt_tracker_sub_cache" => {
            apply_bt_config(engine, key, value).await;
        }
        _ => {}
    }
}

/// Apply proxy config (reads all proxy keys from DB).
async fn apply_proxy_config(engine: &mut Engine, _changed_key: &str, _changed_value: &str) {
    let proxy_mode = engine.db.get_config("proxy_mode").await
        .ok().flatten().unwrap_or_default();
    let proxy_type = engine.db.get_config("proxy_type").await
        .ok().flatten().unwrap_or_default();
    let proxy_host = engine.db.get_config("proxy_host").await
        .ok().flatten().unwrap_or_default();
    let proxy_port = engine.db.get_config("proxy_port").await
        .ok().flatten().unwrap_or_default();
    let proxy_username = engine.db.get_config("proxy_username").await
        .ok().flatten().unwrap_or_default();
    let proxy_password = engine.db.get_config("proxy_password").await
        .ok().flatten().unwrap_or_default();
    let proxy_no_list = engine.db.get_config("proxy_no_list").await
        .ok().flatten().unwrap_or_default();

    let mode = match proxy_mode.as_str() {
        "system" => ProxyMode::System,
        "manual" => ProxyMode::Manual,
        _ => ProxyMode::None,
    };
    let ptype = match proxy_type.as_str() {
        "socks4" => ProxyType::Socks4,
        "socks5" => ProxyType::Socks5,
        "https" => ProxyType::Https,
        _ => ProxyType::Http,
    };
    let port = proxy_port.parse::<u16>().unwrap_or(0);

    let config = ProxyConfig {
        mode,
        proxy_type: ptype,
        host: proxy_host,
        port,
        username: proxy_username,
        password: proxy_password,
        no_proxy_list: proxy_no_list,
    };
    let _ = engine.manager.set_proxy_config(config);
}

/// Apply BT config (reads all BT keys from DB).
async fn apply_bt_config(engine: &mut Engine, _changed_key: &str, _changed_value: &str) {
    let enable_dht = engine.db.get_config("bt_enable_dht").await
        .ok().flatten().map(|v| v == "true").unwrap_or(true);
    let enable_upnp = engine.db.get_config("bt_enable_upnp").await
        .ok().flatten().map(|v| v == "true").unwrap_or(true);
    let port_start = engine.db.get_config("bt_port_start").await
        .ok().flatten().and_then(|v| v.parse::<u16>().ok()).unwrap_or(6881);
    let port_end = engine.db.get_config("bt_port_end").await
        .ok().flatten().and_then(|v| v.parse::<u16>().ok()).unwrap_or(6891);
    let custom_trackers = engine.db.get_config("bt_custom_trackers").await
        .ok().flatten().unwrap_or_default();
    let sub_trackers = engine.db.get_config("bt_tracker_sub_cache").await
        .ok().flatten().unwrap_or_default();

    let config = BtConfig {
        enable_dht,
        enable_upnp,
        port_start,
        port_end,
        custom_trackers,
        subscription_trackers: sub_trackers,
    };
    engine.manager.set_bt_config(config);
}

/// Apply all settings from DB to the engine at startup.
pub async fn apply_all(engine: &mut Engine) {
    // Proxy (reads from DB internally)
    apply_proxy_config(engine, "", "").await;
    // BT (reads from DB internally)
    apply_bt_config(engine, "", "").await;

    if let Ok(Some(v)) = engine.db.get_config("max_concurrent_tasks").await {
        engine.manager.set_max_concurrent(str_to_usize(&v, 5)).await;
    }
    if let Ok(Some(v)) = engine.db.get_config("speed_limit_bytes").await {
        engine.manager.set_speed_limit(str_to_u64(&v, 0));
    }
    if let Ok(Some(v)) = engine.db.get_config("max_auto_retries").await {
        engine.manager.set_max_auto_retries(str_to_i32(&v, 3));
    }
    if let Ok(Some(v)) = engine.db.get_config("auto_retry_delay_secs").await {
        engine.manager.set_auto_retry_delay_secs(str_to_u64(&v, 5));
    }
    if let Ok(Some(v)) = engine.db.get_config("default_save_dir").await {
        engine.manager.set_default_save_dir(v);
    }
    if let Ok(Some(v)) = engine.db.get_config("default_segments").await {
        engine.manager.set_default_segments(str_to_i32(&v, 0));
    }
    if let Ok(Some(v)) = engine.db.get_config("auto_max_connections").await {
        engine.manager.set_auto_max_connections(str_to_i32(&v, 16));
    }
    if let Ok(Some(v)) = engine.db.get_config("use_server_time").await {
        engine.manager.set_use_server_time(str_to_bool(&v, false));
    }
    if let Ok(Some(v)) = engine.db.get_config("global_user_agent").await {
        let _ = engine.manager.set_user_agent(v);
    }
}
