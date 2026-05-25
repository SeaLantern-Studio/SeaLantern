//! `sculk://` 票据格式的编解码。
//!
//! 票据格式：
//! - `"sculk://<EndpointId>?relay=<RelayUrl>"` — 自定义 relay
//! - `"sculk://<EndpointId>"` — 默认 n0 relay

use std::fmt;
use std::str::FromStr;

use crate::error::TicketError;
use crate::types::RelayUrl;
use iroh::EndpointId;

const SCHEME: &str = "sculk";

/// 连接票据，包含目标节点与可选 relay 地址。
#[derive(Debug)]
pub struct Ticket {
    pub endpoint_id: EndpointId,
    pub relay_url: Option<RelayUrl>,
}

impl Ticket {
    /// 创建票据。
    pub fn new(endpoint_id: EndpointId, relay_url: Option<RelayUrl>) -> Self {
        Self {
            endpoint_id,
            relay_url,
        }
    }
}

impl fmt::Display for Ticket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.relay_url {
            Some(relay) => write!(f, "{SCHEME}://{}?relay={relay}", self.endpoint_id),
            None => write!(f, "{SCHEME}://{}", self.endpoint_id),
        }
    }
}

impl FromStr for Ticket {
    type Err = TicketError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let url = url::Url::parse(s)?;

        if url.scheme() != SCHEME {
            return Err(TicketError::InvalidScheme {
                expected: SCHEME,
                actual: url.scheme().to_string(),
            });
        }

        let host = url.host_str().ok_or(TicketError::MissingEndpointId)?;

        if host.is_empty() {
            return Err(TicketError::MissingEndpointId);
        }

        let endpoint_id: EndpointId = host
            .parse::<EndpointId>()
            .map_err(|e| TicketError::EndpointIdParse(e.to_string()))?;

        let relay_url = url
            .query_pairs()
            .find(|(k, _)| k == "relay")
            .map(|(_, v)| v.parse::<RelayUrl>())
            .transpose()
            .map_err(|e| TicketError::RelayUrlParse(e.to_string()))?;

        Ok(Self {
            endpoint_id,
            relay_url,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_endpoint_id() -> EndpointId {
        let bytes: [u8; 32] = rand::random();
        iroh::SecretKey::from_bytes(&bytes).public()
    }

    #[test]
    fn roundtrip_with_relay() {
        let id = test_endpoint_id();
        let relay_res = "https://my-relay.example.com".parse::<RelayUrl>();
        assert!(relay_res.is_ok(), "parse relay failed");
        let relay = if let Ok(v) = relay_res { v } else { return };
        let ticket = Ticket::new(id, Some(relay.clone()));

        let s = ticket.to_string();
        assert!(s.starts_with("sculk://"));
        assert!(s.contains("relay="));

        let parsed_res: std::result::Result<Ticket, TicketError> = s.parse();
        assert!(parsed_res.is_ok(), "parse ticket failed");
        let parsed = if let Ok(v) = parsed_res { v } else { return };
        assert_eq!(parsed.endpoint_id, id);
        assert_eq!(parsed.relay_url.as_ref(), Some(&relay));

        let s2 = parsed.to_string();
        let reparsed_res: std::result::Result<Ticket, TicketError> = s2.parse();
        assert!(reparsed_res.is_ok(), "reparse ticket failed");
        let reparsed = if let Ok(v) = reparsed_res {
            v
        } else {
            return;
        };
        assert_eq!(reparsed.endpoint_id, id);
        assert_eq!(reparsed.relay_url.as_ref(), Some(&relay));
    }

    #[test]
    fn roundtrip_without_relay() {
        let id = test_endpoint_id();
        let ticket = Ticket::new(id, None);

        let s = ticket.to_string();
        assert!(s.starts_with("sculk://"));
        assert!(!s.contains("relay="));

        let parsed_res: std::result::Result<Ticket, TicketError> = s.parse();
        assert!(parsed_res.is_ok(), "parse ticket failed");
        let parsed = if let Ok(v) = parsed_res { v } else { return };
        assert_eq!(parsed.endpoint_id, id);
        assert!(parsed.relay_url.is_none());

        let s2 = parsed.to_string();
        let reparsed_res: std::result::Result<Ticket, TicketError> = s2.parse();
        assert!(reparsed_res.is_ok(), "reparse ticket failed");
        let reparsed = if let Ok(v) = reparsed_res {
            v
        } else {
            return;
        };
        assert_eq!(reparsed.endpoint_id, id);
        assert!(reparsed.relay_url.is_none());
    }

    #[test]
    fn reject_bad_scheme() {
        let result = "http://abc".parse::<Ticket>();
        assert!(result.is_err());
        let err = if let Err(e) = result {
            e.to_string()
        } else {
            return;
        };
        assert!(err.contains("invalid scheme"), "unexpected error: {err}");
    }

    #[test]
    fn reject_missing_host() {
        let result = "sculk:///".parse::<Ticket>();
        assert!(result.is_err());
    }
}
