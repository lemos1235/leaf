#![allow(unused_variables)]

use std::io::Write;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::process::Command;

use anyhow::Result;

use crate::option;

pub fn get_default_ipv4_gateway() -> Result<String> {
    let cols = get_default_ipv4_route_entry()?;
    assert_eq!(cols.len(), 6);
    Ok(cols[5].to_string())
}

pub fn get_default_ipv6_gateway() -> Result<String> {
    let cols = get_default_ipv6_route_entry()?;
    assert_eq!(cols.len(), 6);
    Ok(cols[5].to_string())
}

pub fn get_default_ipv4_address() -> Result<String> {
    todo!()
}

pub fn get_default_ipv6_address() -> Result<String> {
    todo!()
}

pub fn get_ipv4_address(interface: &str) -> Option<String> {
    let out = Command::new("netsh")
        .arg("interface")
        .arg("ipv4")
        .arg("show")
        .arg("ipaddress")
        .output()
        .expect("failed to execute command");
    assert!(out.status.success());
    let output = String::from_utf8_lossy(&out.stdout).to_string();
    find_ip_address(interface, output)
}

pub fn get_ipv6_address(interface: &str) -> Option<String> {
    let out = Command::new("netsh")
        .arg("interface")
        .arg("ipv6")
        .arg("show")
        .arg("ipaddress")
        .output()
        .expect("failed to execute command");
    assert!(out.status.success());
    let output = String::from_utf8_lossy(&out.stdout).to_string();
    find_ip_address(interface, output)
}

fn find_ip_address(interface: &str, output: String) -> Option<String> {
    let addr = output
        .lines()
        .filter(|&line| !line.trim().is_empty())
        .position(|line| line.contains(interface))
        .map(|interface_pos| {
            let lines = output
                .lines()
                .filter(|&line| !line.trim().is_empty())
                .collect::<Vec<_>>();
            let line: &str = &lines[interface_pos + 3];
            return line
                .split_whitespace()
                .map(str::trim)
                .map(str::to_string)
                .last()
                .unwrap();
        });
    addr
}

pub fn get_default_interface() -> Result<String> {
    let if_idx = get_default_ipv4_interface_index()?;
    let out = Command::new("netsh")
        .arg("interface")
        .arg("ipv4")
        .arg("show")
        .arg("interface")
        .output()
        .expect("failed to execute command");
    assert!(out.status.success());
    let output = String::from_utf8_lossy(&out.stdout).to_string();
    let cols: Vec<&str> = output
        .lines()
        .skip(3)
        .map(|line| {
            let a: Vec<&str> = line.split_whitespace().map(str::trim).collect();
            a
        })
        .find(|cols| cols[0] == if_idx.as_str())
        .unwrap();
    assert_eq!(cols.len(), 5);
    Ok(cols[4].to_string())
}

pub fn add_interface_ipv4_address(
    name: &str,
    addr: Ipv4Addr,
    gw: Ipv4Addr,
    mask: Ipv4Addr,
) -> Result<()> {
    let out = Command::new("netsh")
        .arg("interface")
        .arg("ipv4")
        .arg("set")
        .arg("address")
        .arg(name)
        .arg("static")
        .arg(addr.to_string())
        .arg(mask.to_string())
        .arg(gw.to_string())
        .arg("store=active")
        .output()
        .expect("failed to execute command");
    std::io::stdout().write(&out.stdout)?;
    Ok(())
}

pub fn add_interface_ipv6_address(name: &str, addr: Ipv6Addr, prefixlen: i32) -> Result<()> {
    let out = Command::new("netsh")
        .arg("interface")
        .arg("ipv6")
        .arg("set")
        .arg("address")
        .arg(format!("interface={}", name))
        .arg(format!("address={}", addr.to_string()))
        .arg("store=active")
        .output()
        .expect("failed to execute command");
    Ok(())
}

pub fn add_default_ipv4_route(gateway: Ipv4Addr, interface: String, is_tun: bool) -> Result<()> {
    if is_tun {
        let tun_if_idx = get_interface_index(&*option::DEFAULT_TUN_NAME)?;
        Command::new("netsh")
            .arg("interface")
            .arg("ipv4")
            .arg("add")
            .arg("route")
            .arg("0.0.0.0/0")
            .arg(tun_if_idx)
            .arg(gateway.to_string())
            .arg("metric=0")
            .arg("store=active")
            .output()
            .expect("failed to execute command");
    } else {
        let if_idx = get_interface_index(interface.as_str())?;
        Command::new("netsh")
            .arg("interface")
            .arg("ipv4")
            .arg("add")
            .arg("route")
            .arg("0.0.0.0/0")
            .arg(if_idx)
            .arg(gateway.to_string())
            .output()
            .expect("failed to execute command");
    }
    Ok(())
}

pub fn add_default_ipv6_route(gateway: Ipv6Addr, interface: String, is_tun: bool) -> Result<()> {
    if is_tun {
        let tun_if_idx = get_interface_index(&*option::DEFAULT_TUN_NAME)?;
        Command::new("netsh")
            .arg("interface")
            .arg("ipv6")
            .arg("add")
            .arg("route")
            .arg("::/0")
            .arg(tun_if_idx)
            .arg(gateway.to_string())
            .arg("metric=0")
            .arg("store=active")
            .output()
            .expect("failed to execute command");
    } else {
        let if_idx = get_interface_index(interface.as_str())?;
        Command::new("netsh")
            .arg("interface")
            .arg("ipv6")
            .arg("add")
            .arg("route")
            .arg("::/0")
            .arg(if_idx)
            .arg(gateway.to_string())
            .arg("store=active")
            .output()
            .expect("failed to execute command");
    }
    Ok(())
}

pub fn delete_default_ipv6_route(ifscope: Option<String>) -> Result<()> {
    if let Some(scope) = ifscope {
        let if_idx = get_interface_index(scope.as_str())?;
        let out = Command::new("netsh")
            .arg("interface")
            .arg("ipv4")
            .arg("delete")
            .arg("route")
            .arg("::/0")
            .arg("if")
            .arg(if_idx)
            .arg("store=active")
            .output()
            .expect("failed to execute command");
    } else {
        let out = Command::new("route")
            .arg("-6")
            .arg("delete")
            .arg("::/0")
            .output()
            .expect("failed to execute command");
    }
    Ok(())
}

pub fn delete_default_ipv4_route(ifscope: Option<String>) -> Result<()> {
    if let Some(scope) = ifscope {
        let if_idx = get_interface_index(scope.as_str())?;
        let out = Command::new("netsh")
            .arg("interface")
            .arg("ipv4")
            .arg("delete")
            .arg("route")
            .arg("0.0.0.0/0")
            .arg("if")
            .arg(if_idx)
            .arg("store=active")
            .output()
            .expect("failed to execute command");
    } else {
        let out = Command::new("route")
            .arg("-4")
            .arg("delete")
            .arg("0.0.0.0/0")
            .output()
            .expect("failed to execute command");
    }
    Ok(())
}

pub fn add_default_ipv4_rule(addr: Ipv4Addr) -> Result<()> {
    Ok(())
}

pub fn add_default_ipv6_rule(addr: Ipv6Addr) -> Result<()> {
    Ok(())
}

pub fn delete_default_ipv4_rule(addr: Ipv4Addr) -> Result<()> {
    Ok(())
}

pub fn delete_default_ipv6_rule(addr: Ipv6Addr) -> Result<()> {
    Ok(())
}

pub fn get_ipv4_forwarding() -> Result<bool> {
    Ok(false)
}

pub fn get_ipv6_forwarding() -> Result<bool> {
    Ok(false)
}

pub fn set_ipv4_forwarding(val: bool) -> Result<()> {
    //  "HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters"
    todo!()
}

pub fn set_ipv6_forwarding(val: bool) -> Result<()> {
    todo!()
}

pub fn get_default_ipv4_route_entry() -> Result<Vec<String>> {
    let entries = get_ipv4_route_entries()?;
    let e = entries
        .iter()
        .filter(|&e| e[3] == "0.0.0.0/0")
        .last()
        .unwrap();
    Ok(e.clone())
}

pub fn get_interface_index(interface: &str) -> Result<String> {
    let col = get_interface_entry(interface)?;
    Ok(col[0].clone())
}

pub fn get_default_ipv4_interface_index() -> Result<String> {
    let cols = get_default_ipv4_route_entry()?;
    assert_eq!(cols.len(), 6);
    Ok(cols[4].to_string())
}

pub fn get_default_ipv6_route_entry() -> Result<Vec<String>> {
    let entries = get_ipv6_route_entries()?;
    let e = entries.iter().filter(|&e| e[3] == "::/0").last().unwrap();
    Ok(e.clone())
}

pub fn get_interface_entry(interface: &str) -> Result<Vec<String>> {
    let entries = get_interface_entries()?;
    let entry = entries
        .iter()
        .filter(|&e| e[4].eq(interface))
        .last()
        .unwrap()
        .clone();
    Ok(entry)
}

pub fn get_interface_indices() -> Result<Vec<String>> {
    let entires = get_interface_entries()?;
    let indices = entires.iter().map(|e| e[0].to_string()).collect();
    Ok(indices)
}

pub fn get_interface_entries() -> Result<Vec<Vec<String>>> {
    let out = Command::new("netsh")
        .arg("interface")
        .arg("ip")
        .arg("show")
        .arg("interface")
        .output()
        .expect("failed to execute command");
    assert!(out.status.success());
    let output = String::from_utf8_lossy(&out.stdout).to_string();
    let cols = output
        .lines()
        .skip(3)
        .filter(|&line| !line.trim().is_empty())
        .map(|line| {
            let a: Vec<String> = line
                .split_whitespace()
                .map(str::trim)
                .map(str::to_string)
                .collect();
            a
        })
        .collect();
    Ok(cols)
}

pub fn get_ipv4_route_entries() -> Result<Vec<Vec<String>>> {
    let out = Command::new("netsh")
        .arg("interface")
        .arg("ipv4")
        .arg("show")
        .arg("route")
        .output()
        .expect("failed to execute command");
    assert!(out.status.success());
    let out = String::from_utf8_lossy(&out.stdout).to_string();
    get_ip_route_entries(out)
}

pub fn get_ipv6_route_entries() -> Result<Vec<Vec<String>>> {
    let out = Command::new("netsh")
        .arg("interface")
        .arg("ipv6")
        .arg("show")
        .arg("route")
        .output()
        .expect("failed to execute command");
    assert!(out.status.success());
    let out = String::from_utf8_lossy(&out.stdout).to_string();
    get_ip_route_entries(out)
}

fn get_ip_route_entries(out: String) -> Result<Vec<Vec<String>>> {
    let entries = out
        .lines()
        .skip(3)
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.split_whitespace()
                .map(str::trim)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    Ok(entries)
}
