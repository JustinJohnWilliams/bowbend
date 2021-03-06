use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use ::safer_ffi::prelude::*;
use bowbend_core::target::Target as InternalTarget;
use ipnet::{IpNet, Ipv4Net, Ipv6Net};
use safer_ffi::{char_p::char_p_boxed, slice::slice_ref, string::str_ref};

use crate::result::{FfiResult, StatusCodes};

/// This is only used inside `Target` to tag type of the contents.  It's
/// just a workaround for the limitations of enums across the FFI boundary and
/// shouldn't have much use outside `Target`
#[derive_ReprC]
#[repr(u8)]
#[derive(Clone, Debug)]
pub enum TargetType {
    IPv4,
    IPv6,
    IPv4Network,
    IPv6Network,
    Hostname,
}

#[derive_ReprC]
#[ReprC::opaque]
#[derive(Clone, Debug)]
pub struct Target {
    target_type: TargetType,
    contents: Vec<u8>,
}

/// Construct a new `Target` containing an IPv4 address.  If the input
/// slice is anything besides 4 bytes then it will return an error result.
#[ffi_export]
pub fn new_ip_v4_address(input: slice_ref<u8>) -> FfiResult<Target> {
    if input.len() == 4 {
        FfiResult {
            status_code: StatusCodes::Ok,
            contents: Some(repr_c::Box::new(Target {
                target_type: TargetType::IPv4,
                contents: input.to_vec(),
            })),
        }
    } else {
        FfiResult {
            status_code: StatusCodes::InvalidLength,
            contents: None,
        }
    }
}

/// Construct a new `Target` containing an IPv6 address.  If the input
/// slice is anything besides 16 bytes then it will return an error result.
#[ffi_export]
fn new_ip_v6_address(input: slice_ref<u8>) -> FfiResult<Target> {
    if input.len() == 16 {
        FfiResult {
            status_code: StatusCodes::Ok,
            contents: Some(repr_c::Box::new(Target {
                target_type: TargetType::IPv6,
                contents: input.to_vec(),
            })),
        }
    } else {
        FfiResult {
            status_code: StatusCodes::InvalidLength,
            contents: None,
        }
    }
}

#[ffi_export]
fn new_ip_v4_network(address: slice_ref<u8>, prefix: u8) -> FfiResult<Target> {
    if address.len() == 4 && prefix <= 32 {
        let input = vec![address[0], address[1], address[2], address[3], prefix];
        FfiResult {
            status_code: StatusCodes::Ok,
            contents: Some(repr_c::Box::new(Target {
                target_type: TargetType::IPv4Network,
                contents: input,
            })),
        }
    } else {
        FfiResult {
            status_code: StatusCodes::InvalidLength,
            contents: None,
        }
    }
}

#[ffi_export]
fn new_ip_v6_network(address: slice_ref<u8>, prefix: u8) -> FfiResult<Target> {
    if address.len() == 16 && prefix <= 128 {
        let mut buffer = address.to_vec();
        buffer.push(prefix);
        FfiResult {
            status_code: StatusCodes::Ok,
            contents: Some(repr_c::Box::new(Target {
                target_type: TargetType::IPv6Network,
                contents: buffer,
            })),
        }
    } else {
        FfiResult {
            status_code: StatusCodes::InvalidLength,
            contents: None,
        }
    }
}

#[ffi_export]
fn new_hostname(hostname: str_ref) -> FfiResult<Target> {
    // We are pretty liberal about what we are willing to attempt a DNS look up on.
    // So we aren't really going to do anytime of validation on hostnames
    let bytes = hostname.as_bytes();
    if std::str::from_utf8(bytes).is_ok() {
        FfiResult {
            status_code: StatusCodes::Ok,
            contents: Some(repr_c::Box::new(Target {
                target_type: TargetType::Hostname,
                contents: bytes.to_vec(),
            })),
        }
    } else {
        FfiResult {
            status_code: StatusCodes::InvalidUTF8,
            contents: None,
        }
    }
}

#[ffi_export]
fn display_target(target: &Target) -> char_p_boxed {
    let internal_target: InternalTarget = target.clone().into();
    let s = format!("{}", internal_target);
    let x = s.try_into();
    x.unwrap()
}

impl From<Target> for InternalTarget {
    fn from(ffi_target: Target) -> Self {
        match ffi_target.target_type {
            TargetType::IPv4 => InternalTarget::IP(IpAddr::V4(Ipv4Addr::new(
                ffi_target.contents[0],
                ffi_target.contents[1],
                ffi_target.contents[2],
                ffi_target.contents[3],
            ))),
            TargetType::IPv6 => {
                let v: [u8; 16] = ffi_target
                    .contents.
                    try_into()
                    .unwrap_or_else(|v: Vec<u8>| panic!("We reached an invalid internal state by having an IPv6 address of only {} bytes after the length check", v.len()));
                InternalTarget::IP(IpAddr::V6(Ipv6Addr::from(v)))
            }
            TargetType::IPv4Network => {
                let address = Ipv4Addr::new(
                    ffi_target.contents[0],
                    ffi_target.contents[1],
                    ffi_target.contents[2],
                    ffi_target.contents[3],
                );
                // The error state here is triggered by prefixes > 32.  We already check that in
                // the constructor so the unwrap here is safe.
                let network = Ipv4Net::new(address, ffi_target.contents[4]).unwrap();
                InternalTarget::Network(IpNet::V4(network))
            }
            TargetType::IPv6Network => {
                let mut contents = ffi_target.contents;
                let prefix = contents.pop().unwrap();
                let v: [u8; 16] = contents.
                    try_into()
                    .unwrap_or_else(|v: Vec<u8>| panic!("We reached an invalid internal state by having an IPv6 network address of only {} bytes after the length check", v.len()));
                let address = Ipv6Addr::from(v);
                // The error state here is triggered by prefixes > 128.  We already check that
                // in the constructor so the unwrap here is safe
                InternalTarget::Network(IpNet::V6(Ipv6Net::new(address, prefix).unwrap()))
            }
            TargetType::Hostname => {
                // This unwrap is safe due to the check in the constructor
                let hostname_str = std::str::from_utf8(&ffi_target.contents)
                    .unwrap()
                    .to_owned();
                InternalTarget::Hostname(hostname_str)
            }
        }
    }
}
