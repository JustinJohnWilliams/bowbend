from typing import Any, Union

from ._utils import _bytes_to_slice_ref_unit8_t, _char_star_to_python_string
from .portscanner import lib  # type: ignore
from ipaddress import IPv4Address, IPv6Address, IPv4Network, IPv6Network


class PortscanTarget:
    __inner: Any

    def __init__(self, target: Union[IPv4Address, IPv6Address, IPv4Network,
                                     IPv6Network, str]) -> None:
        if isinstance(target, IPv4Address):
            address = _bytes_to_slice_ref_unit8_t(target.packed)
            result = lib.new_ip_v4_address(address)
        elif isinstance(target, IPv6Address):
            address = _bytes_to_slice_ref_unit8_t(target.packed)
            result = lib.new_ip_v6_address(address)
        elif isinstance(target, IPv4Network):
            arg1 = _bytes_to_slice_ref_unit8_t(target.network_address.packed)
            arg2 = target.prefixlen
            result = lib.new_ip_v4_network(arg1, arg2)
        elif isinstance(target, IPv6Network):
            arg1 = _bytes_to_slice_ref_unit8_t(target.network_address.packed)
            arg2 = target.prefixlen
            result = lib.new_ip_v6_network(arg1, arg2)
        else:
            raise ValueError("Not a valid type of target")

        if result.status_code == lib.STATUS_CODES_OK:
            self.__inner = result.contents
        else:
            raise ValueError("Failed to build an target")

    def __str__(self) -> str:
        c_str = lib.display_portscan_target(self.__inner)
        return _char_star_to_python_string(c_str)