/*
#[doc = " No error."]
pub const otError_OT_ERROR_NONE: otError = 0;
#[doc = " Operational failed."]
pub const otError_OT_ERROR_FAILED: otError = 1;
#[doc = " Message was dropped."]
pub const otError_OT_ERROR_DROP: otError = 2;
#[doc = " Insufficient buffers."]
pub const otError_OT_ERROR_NO_BUFS: otError = 3;
#[doc = " No route available."]
pub const otError_OT_ERROR_NO_ROUTE: otError = 4;
#[doc = " Service is busy and could not service the operation."]
pub const otError_OT_ERROR_BUSY: otError = 5;
#[doc = " Failed to parse message."]
pub const otError_OT_ERROR_PARSE: otError = 6;
#[doc = " Input arguments are invalid."]
pub const otError_OT_ERROR_INVALID_ARGS: otError = 7;
#[doc = " Security checks failed."]
pub const otError_OT_ERROR_SECURITY: otError = 8;
#[doc = " Address resolution requires an address query operation."]
pub const otError_OT_ERROR_ADDRESS_QUERY: otError = 9;
#[doc = " Address is not in the source match table."]
pub const otError_OT_ERROR_NO_ADDRESS: otError = 10;
#[doc = " Operation was aborted."]
pub const otError_OT_ERROR_ABORT: otError = 11;
#[doc = " Function or method is not implemented."]
pub const otError_OT_ERROR_NOT_IMPLEMENTED: otError = 12;
#[doc = " Cannot complete due to invalid state."]
pub const otError_OT_ERROR_INVALID_STATE: otError = 13;
#[doc = " No acknowledgment was received after macMaxFrameRetries (IEEE 802.15.4-2006)."]
pub const otError_OT_ERROR_NO_ACK: otError = 14;
#[doc = " A transmission could not take place due to activity on the channel, i.e., the CSMA-CA mechanism has failed\n (IEEE 802.15.4-2006)."]
pub const otError_OT_ERROR_CHANNEL_ACCESS_FAILURE: otError = 15;
#[doc = " Not currently attached to a Thread Partition."]
pub const otError_OT_ERROR_DETACHED: otError = 16;
#[doc = " FCS check failure while receiving."]
pub const otError_OT_ERROR_FCS: otError = 17;
#[doc = " No frame received."]
pub const otError_OT_ERROR_NO_FRAME_RECEIVED: otError = 18;
#[doc = " Received a frame from an unknown neighbor."]
pub const otError_OT_ERROR_UNKNOWN_NEIGHBOR: otError = 19;
#[doc = " Received a frame from an invalid source address."]
pub const otError_OT_ERROR_INVALID_SOURCE_ADDRESS: otError = 20;
#[doc = " Received a frame filtered by the address filter (allowlisted or denylisted)."]
pub const otError_OT_ERROR_ADDRESS_FILTERED: otError = 21;
#[doc = " Received a frame filtered by the destination address check."]
pub const otError_OT_ERROR_DESTINATION_ADDRESS_FILTERED: otError = 22;
#[doc = " The requested item could not be found."]
pub const otError_OT_ERROR_NOT_FOUND: otError = 23;
#[doc = " The operation is already in progress."]
pub const otError_OT_ERROR_ALREADY: otError = 24;
#[doc = " The creation of IPv6 address failed."]
pub const otError_OT_ERROR_IP6_ADDRESS_CREATION_FAILURE: otError = 26;
#[doc = " Operation prevented by mode flags"]
pub const otError_OT_ERROR_NOT_CAPABLE: otError = 27;
#[doc = " Coap response or acknowledgment or DNS, SNTP response not received."]
pub const otError_OT_ERROR_RESPONSE_TIMEOUT: otError = 28;
#[doc = " Received a duplicated frame."]
pub const otError_OT_ERROR_DUPLICATED: otError = 29;
#[doc = " Message is being dropped from reassembly list due to timeout."]
pub const otError_OT_ERROR_REASSEMBLY_TIMEOUT: otError = 30;
#[doc = " Message is not a TMF Message."]
pub const otError_OT_ERROR_NOT_TMF: otError = 31;
#[doc = " Received a non-lowpan data frame."]
pub const otError_OT_ERROR_NOT_LOWPAN_DATA_FRAME: otError = 32;
#[doc = " The link margin was too low."]
pub const otError_OT_ERROR_LINK_MARGIN_LOW: otError = 34;
#[doc = " Input (CLI) command is invalid."]
pub const otError_OT_ERROR_INVALID_COMMAND: otError = 35;
#[doc = " Special error code used to indicate success/error status is pending and not yet known."]
pub const otError_OT_ERROR_PENDING: otError = 36;
#[doc = " Request rejected."]
pub const otError_OT_ERROR_REJECTED: otError = 37;
#[doc = " The number of defined errors."]
pub const otError_OT_NUM_ERRORS: otError = 38;
#[doc = " Generic error (should not use)."]
pub const otError_OT_ERROR_GENERIC: otError = 255;
#[doc = " Represents error codes used throughout OpenThread."]
pub type otError = crate::c_types::c_uint;
extern "C" {
    #[doc = " Converts an otError enum into a string.\n\n @param[in]  aError     An otError enum.\n\n @returns  A string representation of an otError."]
    pub fn otThreadErrorToString(aError: otError) -> *const crate::c_types::c_char;
}

*/

use esp_openthread_sys::{bindings::{otError_OT_ERROR_ABORT, otError_OT_ERROR_FAILED, otError_OT_ERROR_NONE}, c_types};

pub struct OtError {
    ot_error: c_types::c_uint,
}

pub enum OtError {
    None,
    Failed,
    Drop,
    NoBufs,
    NoRoute,
    Busy,
    Parse,
    InvalidArgs,
    Security,
    AddressQuery,
    NoAddress,
    Abort,
    NotImplemented,
    InvalidState,
    NoAck,
    ChannelAccessFailure,
    Detached,
    FcsCheckFailure,
    NoFrameReceived,
    UnknownNeighbor,
    InvalidSourceAddress,
    AddressFiltered,



#[doc = " Received a frame filtered by the destination address check."]
pub const otError_OT_ERROR_DESTINATION_ADDRESS_FILTERED: otError = 22;
#[doc = " The requested item could not be found."]
pub const otError_OT_ERROR_NOT_FOUND: otError = 23;
#[doc = " The operation is already in progress."]
pub const otError_OT_ERROR_ALREADY: otError = 24;
#[doc = " The creation of IPv6 address failed."]
pub const otError_OT_ERROR_IP6_ADDRESS_CREATION_FAILURE: otError = 26;
#[doc = " Operation prevented by mode flags"]
pub const otError_OT_ERROR_NOT_CAPABLE: otError = 27;
#[doc = " Coap response or acknowledgment or DNS, SNTP response not received."]
pub const otError_OT_ERROR_RESPONSE_TIMEOUT: otError = 28;
#[doc = " Received a duplicated frame."]
pub const otError_OT_ERROR_DUPLICATED: otError = 29;
#[doc = " Message is being dropped from reassembly list due to timeout."]
pub const otError_OT_ERROR_REASSEMBLY_TIMEOUT: otError = 30;
#[doc = " Message is not a TMF Message."]
pub const otError_OT_ERROR_NOT_TMF: otError = 31;
#[doc = " Received a non-lowpan data frame."]
pub const otError_OT_ERROR_NOT_LOWPAN_DATA_FRAME: otError = 32;
#[doc = " The link margin was too low."]
pub const otError_OT_ERROR_LINK_MARGIN_LOW: otError = 34;
#[doc = " Input (CLI) command is invalid."]
pub const otError_OT_ERROR_INVALID_COMMAND: otError = 35;
#[doc = " Special error code used to indicate success/error status is pending and not yet known."]
pub const otError_OT_ERROR_PENDING: otError = 36;
    Rejected,
}

impl From<otError> for OtError {
    fn from(value: otError) -> Self {
        match value {
            otError_OT_ERROR_NONE => OtError::None,
            otError_OT_ERROR_FAILED => OtError::Failed,
            otError_OT_ERROR_DROP => OtError::Drop,
            otError_OT_ERROR_NO_BUFS => OtError::NoBufs,
            otError_OT_ERROR_NO_ROUTE => OtError::NoRoute,
            otError_OT_ERROR_BUSY => OtError::Busy,
            otError_OT_ERROR_PARSE => OtError::Parse,
            otError_OT_ERROR_INVALID_ARGS => OtError::InvalidArgs,
            otError_OT_ERROR_SECURITY => OtError::Security,
            otError_OT_ERROR_ADDRESS_QUERY => OtError::AddressQuery,
            otError_OT_ERROR_NO_ADDRESS => OtError::NoAddress,
            otError_OT_ERROR_ABORT => OtError::Abort,
            otError_OT_ERROR_NOT_IMPLEMENTED => OtError::NotImplemented,
            otError_OT_ERROR_INVALID_STATE => OtError::InvalidState,
            otError_OT_ERROR_NO_ACK => OtError::NoAck,
            otError_OT_ERROR_CHANNEL_ACCESS_FAILURE => OtError::ChannelAccessFailure,
            otError_OT_ERROR_DETACHED => OtError::Detached,
            otError_OT_ERROR_FCS => OtError::FcsCheckFailed,
            otError_OT_ERROR_NO_FRAME_RECEIVED => OtError::NoFrameReceived,
            otError_OT_ERROR_UNKNOWN_NEIGHBOR => OtError::UnknownNeighbor,
            otError_OT_ERROR_INVALID_SOURCE_ADDRESS => OtError::InvalidSourceAddress,
            otError_OT_ERROR_ADDRESS_FILTERED => OtError::AddressFiltered,

            otError_OT_ERROR_REJECTED => OtError::Rejected,


        }
    }
}

