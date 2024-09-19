use core::ptr::addr_of_mut;

use esp_ieee802154::Config;
use esp_openthread_sys::bindings::{
    __BindgenBitfieldUnit, otError, otError_OT_ERROR_NONE, otError_OT_ERROR_NOT_CAPABLE,
    otInstance, otPlatRadioTxDone, otPlatRadioTxStarted, otRadioFrame, otRadioFrame__bindgen_ty_1,
    otRadioFrame__bindgen_ty_1__bindgen_ty_1, OT_RADIO_FRAME_MAX_SIZE, OT_RADIO_FRAME_MIN_SIZE,
};

use crate::{get_radio_config, platform::CURRENT_INSTANCE, set_radio_config, with_radio};

pub static mut PSDU: [u8; OT_RADIO_FRAME_MAX_SIZE as usize] =
    [0u8; OT_RADIO_FRAME_MAX_SIZE as usize];
pub static mut TRANSMIT_BUFFER: otRadioFrame = otRadioFrame {
    mPsdu: unsafe { addr_of_mut!(PSDU) as *mut u8 },
    mLength: 0,
    mChannel: 0,
    mRadioType: 0,
    mInfo: otRadioFrame__bindgen_ty_1 {
        mTxInfo: otRadioFrame__bindgen_ty_1__bindgen_ty_1 {
            mAesKey: core::ptr::null(),
            mIeInfo: core::ptr::null_mut(),
            mTxDelay: 0,
            mTxDelayBaseTime: 0,
            mMaxCsmaBackoffs: 0,
            mMaxFrameRetries: 0,
            _bitfield_align_1: [0u8; 0],
            _bitfield_1: __BindgenBitfieldUnit::new([0u8; 1]),
            mRxChannelAfterTxDone: 0,
            mTxPower: 0,
            __bindgen_padding_0: [0u8; 3],
        },
    },
};

pub static mut SENT_FRAME_PSDU: [u8; OT_RADIO_FRAME_MAX_SIZE as usize] =
    [0u8; OT_RADIO_FRAME_MAX_SIZE as usize];
static mut SENT_FRAME: otRadioFrame = otRadioFrame {
    mPsdu: unsafe { addr_of_mut!(SENT_FRAME_PSDU) as *mut u8 },
    mLength: 0,
    mChannel: 0,
    mRadioType: 0,
    mInfo: otRadioFrame__bindgen_ty_1 {
        mTxInfo: otRadioFrame__bindgen_ty_1__bindgen_ty_1 {
            mAesKey: core::ptr::null(),
            mIeInfo: core::ptr::null_mut(),
            mTxDelay: 0,
            mTxDelayBaseTime: 0,
            mMaxCsmaBackoffs: 0,
            mMaxFrameRetries: 0,
            _bitfield_align_1: [0u8; 0],
            _bitfield_1: __BindgenBitfieldUnit::new([0u8; 1]),
            mRxChannelAfterTxDone: 0,
            mTxPower: 0,
            __bindgen_padding_0: [0u8; 3],
        },
    },
};

pub static mut ACK_FRAME_PSDU: [u8; OT_RADIO_FRAME_MIN_SIZE as usize] = [0x2, 0x0, 0x0];
static mut ACK_FRAME: otRadioFrame = otRadioFrame {
    mPsdu: unsafe { addr_of_mut!(ACK_FRAME_PSDU) as *mut u8 },
    mLength: OT_RADIO_FRAME_MIN_SIZE as _,
    mChannel: 0,
    mRadioType: 0,
    mInfo: otRadioFrame__bindgen_ty_1 {
        mTxInfo: otRadioFrame__bindgen_ty_1__bindgen_ty_1 {
            mAesKey: core::ptr::null(),
            mIeInfo: core::ptr::null_mut(),
            mTxDelay: 0,
            mTxDelayBaseTime: 0,
            mMaxCsmaBackoffs: 0,
            mMaxFrameRetries: 0,
            _bitfield_align_1: [0u8; 0],
            _bitfield_1: __BindgenBitfieldUnit::new([0u8; 1]),
            mRxChannelAfterTxDone: 0,
            mTxPower: 0,
            __bindgen_padding_0: [0u8; 3],
        },
    },
};

/// Caller is requires to ensure mac arg is sufficient length
#[no_mangle]
pub extern "C" fn otPlatRadioGetIeeeEui64(_instance: *const otInstance, mac: *mut u8) {
    let efuse_mac = esp_hal::efuse::Efuse::get_mac_address();
    efuse_mac.iter().enumerate().for_each(|(idx, &b)| {
        unsafe { mac.add(idx).write_volatile(b) };
    });
}

#[no_mangle]
pub extern "C" fn otPlatRadioGetCaps(instance: *const otInstance) -> u8 {
    log::info!("otPlatRadioGetCaps {:p}", instance);
    0 // Radio supports no capability. See OT_RADIO_CAPS_*
}

#[no_mangle]
pub extern "C" fn otPlatRadioGetTransmitBuffer(instance: *const otInstance) -> *mut otRadioFrame {
    log::info!("otPlatRadioGetTransmitBuffer {:p}", instance);
    unsafe { addr_of_mut!(TRANSMIT_BUFFER) }
}

#[no_mangle]
pub extern "C" fn otPlatRadioEnable(instance: *const otInstance) -> otError {
    log::info!("otPlatRadioEnable {:p}", instance);
    otError_OT_ERROR_NONE
}

#[no_mangle]
pub extern "C" fn otPlatRadioSleep(instance: *const otInstance) -> otError {
    log::info!("otPlatRadioSleep {:p}", instance);
    otError_OT_ERROR_NONE
}

#[no_mangle]
pub extern "C" fn otPlatRadioGetTransmitPower(
    _instance: *mut otInstance,
    power: *mut i8,
) -> otError {
    let config = get_radio_config();
    unsafe { *power = config.txpower };
    otError_OT_ERROR_NONE
}

#[no_mangle]
pub extern "C" fn otPlatRadioDisable(_instance: *const otInstance) -> otError {
    todo!()
}

#[no_mangle]
pub extern "C" fn otPlatRadioSetPromiscuous(_instance: *const otInstance, enable: bool) {
    let config = set_radio_config(Config {
        promiscuous: enable,
        ..get_radio_config()
    });

    with_radio(|radio| {
        radio.set_config(config);
    });
}

#[no_mangle]
pub extern "C" fn otPlatRadioGetRssi(_instance: *const otInstance) -> i8 {
    log::trace!("otPlatRadioGetRssi reporting last rssi from RCV_fRAME");
    let rssi = unsafe { crate::RCV_FRAME.mInfo.mRxInfo.mRssi };
    // If no rcv frame has set rssi yet, or if rssi is not valid,
    // then use a fake value instead of 0
    if rssi == 0 || rssi >= 127 {
        33
    } else {
        rssi
    }
}

#[no_mangle]
pub extern "C" fn otPlatRadioGetReceiveSensitivity(_instance: *const otInstance) -> i8 {
    log::trace!("otPlatRadioGetReceiveSensitivity reporting const defined in ESP-IDF");
    // from https://github.com/espressif/esp-idf/blob/release/v5.3/components/openthread/src/port/esp_openthread_radio.c#L35
    -120
}

#[no_mangle]
pub extern "C" fn otPlatRadioIsEnabled(_instance: *mut otInstance) -> bool {
    // todo
    true
}

#[no_mangle]
pub extern "C" fn otPlatRadioEnergyScan(
    _instance: *const otInstance,
    _channel: u8,
    _duration: u16,
) -> otError {
    otError_OT_ERROR_NOT_CAPABLE
}

#[no_mangle]
pub extern "C" fn otPlatRadioGetPromiscuous(_instance: *const otInstance) -> bool {
    log::info!("otPlatRadioGetPromiscuous");
    get_radio_config().promiscuous
}

#[no_mangle]
pub extern "C" fn otPlatRadioSetExtendedAddress(instance: *const otInstance, address: *const u8) {
    log::info!("otPlatRadioSetExtendedAddress {:p}", instance);
    let ext_addr = u64::from_be_bytes(
        unsafe { core::slice::from_raw_parts(address, 8) }
            .try_into()
            .unwrap(),
    );

    let config = set_radio_config(Config {
        ext_addr: Some(ext_addr),
        ..get_radio_config()
    });

    with_radio(|radio| {
        radio.set_config(config);
    });
}

#[no_mangle]
pub extern "C" fn otPlatRadioSetShortAddress(instance: *const otInstance, address: u16) {
    log::info!("otPlatRadioSetShortAddress {:p} {}", instance, address);

    let config = set_radio_config(Config {
        short_addr: Some(address),
        ..get_radio_config()
    });

    with_radio(|radio| {
        radio.set_config(config);
    });
}

#[no_mangle]
pub extern "C" fn otPlatRadioSetPanId(_instance: *const otInstance, pan_id: u16) {
    log::info!("otPlatRadioSetPanId {pan_id}");

    let config = set_radio_config(Config {
        pan_id: Some(pan_id),
        ..get_radio_config()
    });

    with_radio(|radio| {
        radio.set_config(config);
    });
}

#[no_mangle]
pub extern "C" fn otPlatRadioTransmit(
    instance: *const otInstance,
    frame: *const otRadioFrame,
) -> otError {
    let frame = unsafe { &*frame };
    let data = unsafe { core::slice::from_raw_parts(frame.mPsdu, frame.mLength as usize) };

    log::trace!(
        "otPlatRadioTransmit channel={} {:02x?}",
        frame.mChannel,
        &data
    );

    let config = set_radio_config(Config {
        channel: frame.mChannel,
        ..get_radio_config()
    });

    with_radio(|radio| {
        radio.set_config(config);
        radio.transmit_raw(data).ok();
    });

    unsafe {
        SENT_FRAME_PSDU[..frame.mLength as usize].copy_from_slice(core::slice::from_raw_parts(
            frame.mPsdu,
            frame.mLength as usize,
        ));
        SENT_FRAME = *frame;
        SENT_FRAME.mPsdu = addr_of_mut!(SENT_FRAME_PSDU) as *mut u8;

        otPlatRadioTxStarted(instance as *mut otInstance, core::mem::transmute(frame));
    }

    log::info!("TX done");

    otError_OT_ERROR_NONE
}

#[no_mangle]
pub extern "C" fn otPlatRadioReceive(_instance: *mut otInstance, channel: u8) -> otError {
    log::debug!("otPlatRadioReceive channel = {channel}");

    let config = set_radio_config(Config {
        channel,
        ..get_radio_config()
    });

    with_radio(|radio| {
        radio.set_config(config);
        radio.start_receive();
    });

    otError_OT_ERROR_NONE
}

pub(crate) fn trigger_tx_done() {
    log::warn!("trigger_tx_done");

    unsafe {
        otPlatRadioTxDone(
            CURRENT_INSTANCE as *mut otInstance,
            addr_of_mut!(SENT_FRAME),
            addr_of_mut!(ACK_FRAME),
            otError_OT_ERROR_NONE,
        );
    }
}
