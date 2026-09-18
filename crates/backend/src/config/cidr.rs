use std::fmt;
use std::net::IpAddr;
use std::str::FromStr;

use super::ConfigError;

const DEFAULT_TRUSTED: [&str; 8] = [
    "127.0.0.0/8",
    "10.0.0.0/8",
    "172.16.0.0/12",
    "192.168.0.0/16",
    "169.254.0.0/16",
    "::1/128",
    "fc00::/7",
    "fe80::/10",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cidr {
    network: IpAddr,
    prefix: u8,
}

impl Cidr {
    pub fn contains(&self, ip: IpAddr) -> bool {
        match (self.network, unmap(ip)) {
            (IpAddr::V4(net), IpAddr::V4(ip)) => {
                mask_v4(net.to_bits(), self.prefix) == mask_v4(ip.to_bits(), self.prefix)
            }
            (IpAddr::V6(net), IpAddr::V6(ip)) => {
                mask_v6(net.to_bits(), self.prefix) == mask_v6(ip.to_bits(), self.prefix)
            }
            _ => false,
        }
    }

    pub fn is_private(&self) -> bool {
        default_trusted_proxies().iter().any(|d| d.covers(self))
    }

    fn covers(&self, other: &Self) -> bool {
        self.prefix <= other.prefix && self.contains(other.network)
    }
}

impl fmt::Display for Cidr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.network, self.prefix)
    }
}

impl FromStr for Cidr {
    type Err = ConfigError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let invalid = || ConfigError::InvalidValue {
            key: "TRUSTED_PROXIES".into(),
            value: raw.to_string(),
        };
        let (addr_part, prefix_part) = match raw.split_once('/') {
            Some((addr, prefix)) => (addr, Some(prefix)),
            None => (raw, None),
        };
        let addr: IpAddr = addr_part.parse().map_err(|_| invalid())?;
        let addr = unmap(addr);
        let bits = match addr {
            IpAddr::V4(_) => 32,
            IpAddr::V6(_) => 128,
        };
        let prefix = match prefix_part {
            Some(p) => p.parse::<u8>().map_err(|_| invalid())?,
            None => bits,
        };
        if prefix > bits {
            return Err(invalid());
        }
        let network = match addr {
            IpAddr::V4(v4) => IpAddr::from(mask_v4(v4.to_bits(), prefix).to_be_bytes()),
            IpAddr::V6(v6) => IpAddr::from(mask_v6(v6.to_bits(), prefix).to_be_bytes()),
        };
        Ok(Self { network, prefix })
    }
}

pub fn default_trusted_proxies() -> Vec<Cidr> {
    DEFAULT_TRUSTED
        .iter()
        .map(|raw| raw.parse().expect("built-in CIDR"))
        .collect()
}

fn unmap(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => match v6.to_ipv4_mapped() {
            Some(v4) => IpAddr::V4(v4),
            None => IpAddr::V6(v6),
        },
        other => other,
    }
}

fn mask_v4(bits: u32, prefix: u8) -> u32 {
    match prefix {
        0 => 0,
        p => bits & (u32::MAX << (32 - p)),
    }
}

fn mask_v6(bits: u128, prefix: u8) -> u128 {
    match prefix {
        0 => 0,
        p => bits & (u128::MAX << (128 - p)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cidr(raw: &str) -> Cidr {
        raw.parse().expect(raw)
    }

    #[test]
    fn matches_addresses_inside_the_prefix() {
        let cases = [
            ("10.0.0.0/8", "10.255.3.4", true),
            ("10.0.0.0/8", "11.0.0.1", false),
            ("172.16.0.0/12", "172.31.255.255", true),
            ("172.16.0.0/12", "172.32.0.1", false),
            ("203.0.113.7", "203.0.113.7", true),
            ("203.0.113.7", "203.0.113.8", false),
            ("0.0.0.0/0", "8.8.8.8", true),
            ("fc00::/7", "fd00::1", true),
            ("fc00::/7", "2001:db8::1", false),
            ("::1/128", "::1", true),
        ];
        for (net, ip, expected) in cases {
            if cidr(net).contains(ip.parse().unwrap()) != expected {
                panic!("{net} contains {ip} should be {expected}");
            }
        }
    }

    #[test]
    fn v4_mapped_peers_match_v4_prefixes() {
        if !cidr("10.0.0.0/8").contains("::ffff:10.0.0.1".parse().unwrap()) {
            panic!("v4-mapped peer must match the v4 prefix");
        }
        if cidr("fc00::/7").contains("::ffff:10.0.0.1".parse().unwrap()) {
            panic!("v4-mapped peer must not match a v6 prefix");
        }
    }

    #[test]
    fn a_prefix_is_canonicalised_to_its_network() {
        if cidr("10.1.2.3/8").to_string() != "10.0.0.0/8" {
            panic!("host bits must be cleared");
        }
    }

    #[test]
    fn rejects_malformed_values() {
        for raw in ["10.0.0.0/33", "::1/129", "not-an-ip", "10.0.0.0/x", ""] {
            if raw.parse::<Cidr>().is_ok() {
                panic!("{raw} must not parse");
            }
        }
    }

    #[test]
    fn public_prefixes_are_not_private() {
        if !cidr("192.168.1.0/24").is_private() {
            panic!("192.168.1.0/24 is inside the default private set");
        }
        if cidr("203.0.113.0/24").is_private() {
            panic!("203.0.113.0/24 is public");
        }
        if cidr("0.0.0.0/0").is_private() {
            panic!("a prefix wider than the private set is not private");
        }
    }
}
