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

impl Default for bindings::otMacCounters {
    fn default() -> Self {
        Self {
            mTxTotal: 0,
            mTxUnicast: 0,
            mTxBroadcast: 0,
            mTxAckRequested: 0,
            mTxAcked: 0,
            mTxNoAckRequested: 0,
            mTxData: 0,
            mTxDataPoll: 0,
            mTxBeacon: 0,
            mTxBeaconRequest: 0,
            mTxOther: 0,
            mTxRetry: 0,
            mTxDirectMaxRetryExpiry: 0,
            mTxIndirectMaxRetryExpiry: 0,
            mTxErrCca: 0,
            mTxErrAbort: 0,
            mTxErrBusyChannel: 0,
            mRxTotal: 0,
            mRxUnicast: 0,
            mRxBroadcast: 0,
            mRxData: 0,
            mRxDataPoll: 0,
            mRxBeacon: 0,
            mRxBeaconRequest: 0,
            mRxOther: 0,
            mRxAddressFiltered: 0,
            mRxDestAddrFiltered: 0,
            mRxDuplicated: 0,
            mRxErrNoFrame: 0,
            mRxErrUnknownNeighbor: 0,
            mRxErrInvalidSrcAddr: 0,
            mRxErrSec: 0,
            mRxErrFcs: 0,
            mRxErrOther: 0,
        }
    }
}
