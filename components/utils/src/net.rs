use std::net::IpAddr;
use std::net::TcpListener;

pub fn get_available_port(interface: IpAddr, avoid: u16) -> Option<u16> {
    // Start after "well-known" ports (0–1023) as they require superuser
    // privileges on UNIX-like operating systems.
    (1024..9000).find(|port| *port != avoid && port_is_available(interface, *port))
}

pub fn port_is_available(interface: IpAddr, port: u16) -> bool {
    TcpListener::bind((interface, port)).is_ok()
}

/// Returns whether a link starts with an HTTP(s) scheme.
pub fn is_external_link(link: &str) -> bool {
    link.starts_with("http:") || link.starts_with("https:")
}
