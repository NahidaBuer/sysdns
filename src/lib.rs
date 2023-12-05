use std::net::{SocketAddr, UdpSocket};
use std::{process::Command, str::from_utf8};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SysDNS {
    pub enable: bool,
    pub server: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to parse string `{0}`")]
    ParseStr(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("failed to get default network interface")]
    NetworkInterface,
}

pub type Result<T> = std::result::Result<T, Error>;

impl SysDNS {
    pub fn is_support() -> bool {
        cfg!(any(target_os = "macos",))
    }

    pub fn get_system_dns() -> Result<SysDNS> {
        let service = default_network_service().or_else(|_| default_network_service_by_ns())?;
        let service = service.as_str();

        let dns = get_dns(service)?;

        Ok(dns)
    }

    pub fn set_system_dns(&self) -> Result<()> {
        let service = default_network_service().or_else(|_| default_network_service_by_ns())?;
        let service = service.as_str();

        set_dns(self, service)?;

        Ok(())
    }
}

fn networksetup() -> Command {
    Command::new("networksetup")
}

fn set_dns(dns: &SysDNS, service: &str) -> Result<()> {
    let target = "-setdnsservers";

    let mut server = dns.server.as_str();
    if !dns.enable {
        server = "Empty";
    }

    networksetup().args([target, service, server]).status()?;

    Ok(())
}

fn get_dns(service: &str) -> Result<SysDNS> {
    let target = "-getdnsservers";

    let output = networksetup().args([target, service]).output()?;

    let stdout = from_utf8(&output.stdout).or(Err(Error::ParseStr("output".into())))?;
    let enable = !(stdout.contains("aren't any"));

    let mut server = "Empty".to_owned();
    if enable {
        if let Some(stripped) = stdout.strip_suffix("\n") {
            server = stripped.to_owned();
        }
    } else {
        server = "Empty".to_owned();
    }

    Ok(SysDNS { enable, server })
}

fn default_network_service() -> Result<String> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("1.1.1.1:80")?;
    let ip = socket.local_addr()?.ip();
    let addr = SocketAddr::new(ip, 0);

    let interfaces = interfaces::Interface::get_all().or(Err(Error::NetworkInterface))?;
    let interface = interfaces
        .into_iter()
        .find(|i| i.addresses.iter().find(|a| a.addr == Some(addr)).is_some())
        .map(|i| i.name.to_owned());

    match interface {
        Some(interface) => {
            let service = get_service_by_device(interface)?;
            Ok(service)
        }
        None => Err(Error::NetworkInterface),
    }
}

fn default_network_service_by_ns() -> Result<String> {
    let output = networksetup().arg("-listallnetworkservices").output()?;
    let stdout = from_utf8(&output.stdout).or(Err(Error::ParseStr("output".into())))?;
    let mut lines = stdout.split('\n');
    lines.next(); // ignore the tips

    // get the first service
    match lines.next() {
        Some(line) => Ok(line.into()),
        None => Err(Error::NetworkInterface),
    }
}

fn get_service_by_device(device: String) -> Result<String> {
    let output = networksetup().arg("-listallhardwareports").output()?;
    let stdout = from_utf8(&output.stdout).or(Err(Error::ParseStr("output".into())))?;

    let hardware = stdout.split("Ethernet Address:").find_map(|s| {
        let lines = s.split("\n");
        let mut hardware = None;
        let mut device_ = None;

        for line in lines {
            if line.starts_with("Hardware Port:") {
                hardware = Some(&line[15..]);
            }
            if line.starts_with("Device:") {
                device_ = Some(&line[8..])
            }
        }

        if device == device_? {
            hardware
        } else {
            None
        }
    });

    match hardware {
        Some(hardware) => Ok(hardware.into()),
        None => Err(Error::NetworkInterface),
    }
}
