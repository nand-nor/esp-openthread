#![no_std]
#[allow(improper_ctypes)]
pub mod bindings;
pub mod c_types;

impl core::fmt::Debug for bindings::otNetifAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("otNetifAddress")
            .field("mAddress", &self.mAddress)
            .field("mPrefixLength", &self.mPrefixLength)
            .field("mAddressOrigin", &self.mAddressOrigin)
            .field("_bitfield_align_1", &self._bitfield_align_1)
            .field("_bitfield_1", &self._bitfield_1)
            .field("mNext", &self.mNext)
            .finish()
    }
}

impl core::fmt::Debug for bindings::otIp6Address {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("otIp6Address")
            .field("mFields", &self.mFields)
            .finish()
    }
}

impl core::fmt::Debug for bindings::otIp6Address__bindgen_ty_1 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let ptr = unsafe { core::ptr::addr_of!(self.m16) };
        let octets = unsafe { ptr.read_unaligned() };
        f.debug_struct("otIp6Address__bindgen_ty_1")
            .field("m16", &octets)
            .finish()
    }
}
