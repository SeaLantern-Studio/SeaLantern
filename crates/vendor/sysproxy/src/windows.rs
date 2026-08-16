use crate::{Autoproxy, Error, Result, Sysproxy};
use std::{ffi::c_void, mem::size_of};
use url::Url;
use windows::{
    Win32::{
        NetworkManagement::Rras::{ERROR_BUFFER_TOO_SMALL, RASENTRYNAMEW, RasEnumEntriesW},
        Networking::WinInet::{
            INTERNET_OPTION_PER_CONNECTION_OPTION, INTERNET_OPTION_PROXY_SETTINGS_CHANGED,
            INTERNET_OPTION_REFRESH, INTERNET_PER_CONN_AUTOCONFIG_URL, INTERNET_PER_CONN_FLAGS,
            INTERNET_PER_CONN_OPTION_LISTW, INTERNET_PER_CONN_OPTIONW, INTERNET_PER_CONN_OPTIONW_0,
            INTERNET_PER_CONN_PROXY_BYPASS, INTERNET_PER_CONN_PROXY_SERVER, InternetSetOptionW,
            PROXY_TYPE_AUTO_DETECT, PROXY_TYPE_AUTO_PROXY_URL, PROXY_TYPE_DIRECT, PROXY_TYPE_PROXY,
        },
    },
    core::{PCWSTR, PWSTR},
};
use winreg::{RegKey, enums};

pub use windows::core::Error as Win32Error;

const SUB_KEY: &str = "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Internet Settings";

fn set_proxy_enable(enable: bool) -> Result<()> {
    let expected = u32::from(enable);
    let hkcu = RegKey::predef(enums::HKEY_CURRENT_USER);
    let cur_var =
        hkcu.open_subkey_with_flags(SUB_KEY, enums::KEY_QUERY_VALUE | enums::KEY_SET_VALUE)?;

    cur_var.set_value("ProxyEnable", &expected)?;
    let actual = cur_var.get_value::<u32, _>("ProxyEnable")?;
    if actual != expected {
        return Err(Error::ProxyEnableMismatch { expected, actual });
    }

    Ok(())
}

fn encode_wide<S: AsRef<std::ffi::OsStr>>(string: S) -> Vec<u16> {
    std::os::windows::prelude::OsStrExt::encode_wide(string.as_ref())
        .chain(std::iter::once(0))
        .collect::<Vec<u16>>()
}

/// unset proxy
///
/// **对于包含中文字符的拨号连接或 VPN 连接，可能无法正确设置其代理，建议使用全英文重命名该连接名称**
#[inline]
fn unset_proxy() -> Result<()> {
    let mut p_opts = Vec::<INTERNET_PER_CONN_OPTIONW>::with_capacity(1);
    p_opts.push(INTERNET_PER_CONN_OPTIONW {
        dwOption: INTERNET_PER_CONN_FLAGS,
        Value: {
            let mut v = INTERNET_PER_CONN_OPTIONW_0::default();
            v.dwValue = PROXY_TYPE_DIRECT;
            v
        },
    });
    let mut opts = INTERNET_PER_CONN_OPTION_LISTW {
        dwSize: size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32,
        dwOptionCount: 1,
        dwOptionError: 0,
        pOptions: p_opts.as_mut_ptr(),
        pszConnection: PWSTR::null(),
    };

    // 局域网 LAN 代理设置
    apply_option(&opts)?;
    // 拨号连接/VPN 代理设置
    let ras_conns = get_ras_connections()?;
    for ras_conn in ras_conns.iter() {
        let conn_wide = encode_wide(ras_conn);
        opts.pszConnection = PWSTR::from_raw(conn_wide.as_ptr() as *mut u16);
        apply_option(&opts)?;
        log::debug!("unset RAS[{ras_conn}] proxy success");
    }
    set_proxy_enable(false)?;
    notify_proxy_change()
}

/// set auto proxy
///
/// **对于包含中文字符的拨号连接或 VPN 连接，可能无法正确设置其代理，建议使用全英文重命名该连接名称**
#[inline]
fn set_auto_proxy(url: &str) -> Result<()> {
    let s = encode_wide(url);
    let mut p_opts = Vec::<INTERNET_PER_CONN_OPTIONW>::with_capacity(2);
    p_opts.push(INTERNET_PER_CONN_OPTIONW {
        dwOption: INTERNET_PER_CONN_FLAGS,
        Value: INTERNET_PER_CONN_OPTIONW_0 {
            dwValue: PROXY_TYPE_AUTO_DETECT | PROXY_TYPE_AUTO_PROXY_URL | PROXY_TYPE_DIRECT,
        },
    });
    p_opts.push(INTERNET_PER_CONN_OPTIONW {
        dwOption: INTERNET_PER_CONN_AUTOCONFIG_URL,
        Value: INTERNET_PER_CONN_OPTIONW_0 {
            pszValue: PWSTR::from_raw(s.as_ptr() as *mut u16),
        },
    });

    let mut opts = INTERNET_PER_CONN_OPTION_LISTW {
        dwSize: size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32,
        dwOptionCount: 2,
        dwOptionError: 0,
        pOptions: p_opts.as_mut_ptr(),
        pszConnection: PWSTR::null(),
    };

    // 局域网 LAN 代理设置
    apply_option(&opts)?;
    // 拨号连接/VPN 代理设置
    let ras_conns = get_ras_connections()?;
    for ras_conn in ras_conns.iter() {
        let conn_wide = encode_wide(ras_conn);
        opts.pszConnection = PWSTR::from_raw(conn_wide.as_ptr() as *mut u16);
        apply_option(&opts)?;
        log::debug!("set RAS[{ras_conn}] auto proxy success");
    }
    set_proxy_enable(false)?;
    notify_proxy_change()
}

/// set global proxy
///
/// **对于包含中文字符的拨号连接或 VPN 连接，可能无法正确设置其代理，建议使用全英文重命名该连接名称**
#[inline]
fn set_global_proxy(server: &str, bypass: &str) -> Result<()> {
    let s = encode_wide(server);
    let b = encode_wide(bypass);
    let mut p_opts = Vec::<INTERNET_PER_CONN_OPTIONW>::with_capacity(3);
    p_opts.push(INTERNET_PER_CONN_OPTIONW {
        dwOption: INTERNET_PER_CONN_FLAGS,
        Value: INTERNET_PER_CONN_OPTIONW_0 {
            dwValue: PROXY_TYPE_PROXY | PROXY_TYPE_DIRECT,
        },
    });
    p_opts.push(INTERNET_PER_CONN_OPTIONW {
        dwOption: INTERNET_PER_CONN_PROXY_SERVER,
        Value: INTERNET_PER_CONN_OPTIONW_0 {
            pszValue: PWSTR::from_raw(s.as_ptr() as *mut u16),
        },
    });
    p_opts.push(INTERNET_PER_CONN_OPTIONW {
        dwOption: INTERNET_PER_CONN_PROXY_BYPASS,
        Value: INTERNET_PER_CONN_OPTIONW_0 {
            pszValue: PWSTR::from_raw(b.as_ptr() as *mut u16),
        },
    });

    let mut opts = INTERNET_PER_CONN_OPTION_LISTW {
        dwSize: size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32,
        dwOptionCount: 3,
        dwOptionError: 0,
        pOptions: p_opts.as_mut_ptr(),
        pszConnection: PWSTR::null(),
    };
    // 局域网 LAN 代理设置
    apply_option(&opts)?;
    // 拨号连接/VPN 代理设置
    let ras_conns = get_ras_connections()?;
    for ras_conn in ras_conns.iter() {
        let conn_wide = encode_wide(ras_conn);
        opts.pszConnection = PWSTR::from_raw(conn_wide.as_ptr() as *mut u16);
        apply_option(&opts)?;
        log::debug!("set RAS[{ras_conn}] global proxy success");
    }
    set_proxy_enable(true)?;
    notify_proxy_change()
}

#[inline]
fn apply_option(options: &INTERNET_PER_CONN_OPTION_LISTW) -> Result<()> {
    unsafe {
        // setting options
        let opts = options as *const INTERNET_PER_CONN_OPTION_LISTW as *const c_void;
        InternetSetOptionW(
            None,
            INTERNET_OPTION_PER_CONNECTION_OPTION,
            Some(opts),
            size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32,
        )?;
    }
    Ok(())
}

#[inline]
fn notify_proxy_change() -> Result<()> {
    unsafe {
        InternetSetOptionW(None, INTERNET_OPTION_PROXY_SETTINGS_CHANGED, None, 0)?;
        // refreshing
        InternetSetOptionW(None, INTERNET_OPTION_REFRESH, None, 0)?;
    }
    Ok(())
}

impl Sysproxy {
    #[inline]
    pub fn get_system_proxy() -> Result<Sysproxy> {
        let hkcu = RegKey::predef(enums::HKEY_CURRENT_USER);
        let cur_var = hkcu.open_subkey_with_flags(SUB_KEY, enums::KEY_QUERY_VALUE)?;
        let enable = cur_var.get_value::<u32, _>("ProxyEnable").unwrap_or(0u32) == 1u32;
        let proxy_server = cur_var
            .get_value::<String, _>("ProxyServer")
            .unwrap_or_default();

        // 预设默认值
        let mut host = String::new();
        let mut port = 0u16;

        if !proxy_server.is_empty() {
            if proxy_server.contains('=') {
                // 处理多协议格式: http=127.0.0.1:7890;https=127.0.0.1:7890
                // 优先查找http代理
                let http_proxy = find_http_proxy(&proxy_server);

                if let Some(proxy) = http_proxy {
                    let proxy_value = proxy.split('=').nth(1).unwrap_or("");
                    parse_proxy_address(proxy_value, &mut host, &mut port);
                } else {
                    return Err(Error::NotSupport);
                }
            } else {
                // 处理单一格式: 127.0.0.1:7890
                parse_proxy_address(&proxy_server, &mut host, &mut port);
            }
        }

        let bypass = cur_var.get_value("ProxyOverride").unwrap_or_default();

        Ok(Sysproxy { enable, host, port, bypass })
    }

    #[inline]
    pub fn set_system_proxy(&self) -> Result<()> {
        match self.enable {
            true => set_global_proxy(&format!("{}:{}", self.host, self.port), &self.bypass),
            false => unset_proxy(),
        }
    }
}

impl Autoproxy {
    #[inline]
    pub fn get_auto_proxy() -> Result<Autoproxy> {
        let hkcu = RegKey::predef(enums::HKEY_CURRENT_USER);
        let cur_var = hkcu.open_subkey_with_flags(SUB_KEY, enums::KEY_QUERY_VALUE)?;
        let url = cur_var.get_value::<String, _>("AutoConfigURL");
        let enable = url.is_ok();
        let url = url.unwrap_or_default();

        Ok(Autoproxy { enable, url })
    }

    #[inline]
    pub fn set_auto_proxy(&self) -> Result<()> {
        match self.enable {
            true => set_auto_proxy(&self.url),
            false => unset_proxy(),
        }
    }
}

fn find_http_proxy(proxy_server: &str) -> Option<&str> {
    proxy_server.split(';').find(|part| {
        let candidate = part.trim().as_bytes();
        candidate.len() >= 5 && candidate[..5].eq_ignore_ascii_case(b"http=")
    })
}

/// 解析代理地址字符串为主机名和端口
#[inline]
fn parse_proxy_address(address: &str, host: &mut String, port: &mut u16) {
    // 快速路径：host:port 或 [ipv6]:port，无需堆分配
    if let Some((h, p)) = address.rsplit_once(':') {
        if let Ok(port_num) = p.parse::<u16>() {
            // 去除 IPv6 方括号: "[::1]" → "::1"
            let clean = if h.starts_with('[') && h.ends_with(']') {
                &h[1..h.len() - 1]
            } else {
                h
            };
            *host = clean.to_string();
            *port = port_num;
            return;
        }
    }

    // 回退：URL 解析器处理无端口的主机名等边缘情况
    if let Ok(url) = Url::parse(&format!("http://{}", address)) {
        *host = url.host_str().unwrap_or("").to_string();
        *port = url.port().unwrap_or(80);
        return;
    }

    // 如果无法解析端口，默认使用主机名和标准HTTP端口
    *host = address.to_string();
    *port = 80;
}

/// refer: https://learn.microsoft.com/zh-cn/windows/win32/api/ras/nf-ras-rasenumentriesw
///
/// 获取所有远程访问服务 （包含拨号连接和 VPN 连接）
fn get_ras_connections() -> Result<Vec<String>> {
    log::debug!("start get RAS connections...");
    let mut connections = Vec::new();

    unsafe {
        let mut buffer_size = 0u32;
        let mut entry_count = 0u32;

        let result_code = RasEnumEntriesW(
            PCWSTR::null(),
            PCWSTR::null(),
            None,
            &mut buffer_size,
            &mut entry_count,
        );

        log::debug!("get allocate buffer size result code: {result_code}");

        if result_code != ERROR_BUFFER_TOO_SMALL {
            if entry_count >= 1 {
                log::error!("The operation failed to acquire the buffer size");
            } else {
                log::debug!("There were no RAS entry names found");
            }
            return Ok(connections);
        }

        let entry_capacity = buffer_size as usize / std::mem::size_of::<RASENTRYNAMEW>();
        if entry_capacity == 0 {
            return Ok(connections);
        }

        // 使用 Vec 管理缓冲区内存，避免手动堆分配
        let mut entries: Vec<RASENTRYNAMEW> = Vec::with_capacity(entry_capacity);
        for _ in 0..entry_capacity {
            entries.push(std::mem::zeroed());
        }
        // The first RASENTRYNAME structure in the array must contain the structure size
        entries[0].dwSize = std::mem::size_of::<RASENTRYNAMEW>() as u32;

        // 获取所有 RAS 列表
        let result_code = RasEnumEntriesW(
            PCWSTR::null(),
            PCWSTR::null(),
            Some(entries.as_mut_ptr()),
            &mut buffer_size,
            &mut entry_count,
        );
        // 如果函数成功，则返回值 ERROR_SUCCESS, 但是该 API 返回 u32, 参照对比 ERROR_SUCCESS 后，该值应该为 0
        log::debug!("get RAS entries result code: {result_code}");

        if result_code == 0 && entry_count > 0 {
            for entry in entries.iter().take(entry_count as usize) {
                let name_arr = entry.szEntryName;
                // 去除宽字符多余的 0，以便更好的打印 RAS 名称
                let len = name_arr.iter().position(|&x| x == 0).unwrap_or(0);
                let name = String::from_utf16_lossy(&name_arr[..len]);
                connections.push(name);
            }
            log::debug!("找到 {} 个拨号连接/VPN, {:?}", connections.len(), connections);
        }
    }

    Ok(connections)
}

#[cfg(test)]
mod tests {
    use super::{find_http_proxy, parse_proxy_address};

    fn parse(addr: &str) -> (String, u16) {
        let mut host = String::new();
        let mut port = 0u16;
        parse_proxy_address(addr, &mut host, &mut port);
        (host, port)
    }

    #[test]
    fn test_ipv4_with_port() {
        assert_eq!(parse("127.0.0.1:8080"), ("127.0.0.1".into(), 8080));
    }

    #[test]
    fn test_hostname_with_port() {
        assert_eq!(parse("proxy.example.com:3128"), ("proxy.example.com".into(), 3128));
    }

    #[test]
    fn test_ipv6_bracketed_with_port() {
        assert_eq!(parse("[::1]:1080"), ("::1".into(), 1080));
    }

    #[test]
    fn test_hostname_only_defaults_port_80() {
        assert_eq!(parse("proxy.example.com"), ("proxy.example.com".into(), 80));
    }

    #[test]
    fn test_ipv4_only_defaults_port_80() {
        assert_eq!(parse("192.168.1.1"), ("192.168.1.1".into(), 80));
    }

    #[test]
    fn test_empty_string() {
        let (host, port) = parse("");
        assert_eq!(port, 80);
        assert!(host.is_empty());
    }

    #[test]
    fn test_high_port() {
        assert_eq!(parse("10.0.0.1:65535"), ("10.0.0.1".into(), 65535));
    }

    #[test]
    fn socks_only_mapping_is_not_an_http_proxy() {
        assert!(find_http_proxy("socks=127.0.0.1:1080").is_none());
        assert_eq!(
            find_http_proxy("socks=127.0.0.1:1080;HTTP=127.0.0.1:7890"),
            Some("HTTP=127.0.0.1:7890")
        );
    }
}
