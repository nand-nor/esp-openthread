#![no_std]
#![feature(c_variadic)]

#[cfg(any(
    all(feature = "mtd", feature = "sed"),
    all(feature = "mtd", feature = "ssed"),
    all(feature = "mtd", feature = "ftd"),
    all(feature = "ftd", feature = "sed"),
    all(feature = "ftd", feature = "ssed"),
))]
compile_error!("Multiple device type features set");

#[cfg(any(feature = "ftd", feature = "sed", feature = "ssed"))]
compile_error!("Unsupported device type selected");

mod entropy;
mod net;
mod platform;
mod radio;
mod timer;

pub use net::udp::{udp_receive_handler, UdpSocket};

use core::{borrow::BorrowMut, cell::RefCell, marker::PhantomData, ptr::addr_of_mut};

use bitflags::bitflags;
use critical_section::Mutex;
use esp_hal::timer::systimer::{Alarm, Target};
use esp_ieee802154::{rssi_to_lqi, Config, Ieee802154};

// for now just re-export all
pub use esp_openthread_sys as sys;
use sys::bindings::{
    otChangedFlags, otDatasetSetActive, otDeviceRole, otDeviceRole_OT_DEVICE_ROLE_CHILD,
    otDeviceRole_OT_DEVICE_ROLE_DETACHED, otDeviceRole_OT_DEVICE_ROLE_DISABLED,
    otDeviceRole_OT_DEVICE_ROLE_LEADER, otDeviceRole_OT_DEVICE_ROLE_ROUTER, otError_OT_ERROR_NONE,
    otExtendedPanId, otInstance, otInstanceInitSingle, otIp6GetUnicastAddresses, otIp6SetEnabled,
    otLinkModeConfig, otMeshLocalPrefix, otNetworkKey, otNetworkName, otOperationalDataset,
    otOperationalDatasetComponents, otPlatRadioGetIeeeEui64, otPlatRadioReceiveDone, otPskc,
    otSecurityPolicy, otSetStateChangedCallback, otTaskletsArePending, otTaskletsProcess,
    otThreadGetDeviceRole, otThreadGetLinkMode, otThreadSetChildTimeout, otThreadSetEnabled,
    otThreadSetLinkMode, otTimestamp, OT_CHANGED_ACTIVE_DATASET,
    OT_CHANGED_CHANNEL_MANAGER_NEW_CHANNEL, OT_CHANGED_COMMISSIONER_STATE,
    OT_CHANGED_IP6_ADDRESS_ADDED, OT_CHANGED_IP6_ADDRESS_REMOVED,
    OT_CHANGED_IP6_MULTICAST_SUBSCRIBED, OT_CHANGED_IP6_MULTICAST_UNSUBSCRIBED,
    OT_CHANGED_JOINER_STATE, OT_CHANGED_NAT64_TRANSLATOR_STATE, OT_CHANGED_NETWORK_KEY,
    OT_CHANGED_PARENT_LINK_QUALITY, OT_CHANGED_PENDING_DATASET, OT_CHANGED_PSKC,
    OT_CHANGED_SECURITY_POLICY, OT_CHANGED_SUPPORTED_CHANNEL_MASK,
    OT_CHANGED_THREAD_BACKBONE_ROUTER_LOCAL, OT_CHANGED_THREAD_BACKBONE_ROUTER_STATE,
    OT_CHANGED_THREAD_CHANNEL, OT_CHANGED_THREAD_CHILD_ADDED, OT_CHANGED_THREAD_CHILD_REMOVED,
    OT_CHANGED_THREAD_EXT_PANID, OT_CHANGED_THREAD_KEY_SEQUENCE_COUNTER, OT_CHANGED_THREAD_LL_ADDR,
    OT_CHANGED_THREAD_ML_ADDR, OT_CHANGED_THREAD_NETDATA, OT_CHANGED_THREAD_NETIF_STATE,
    OT_CHANGED_THREAD_NETWORK_NAME, OT_CHANGED_THREAD_PANID, OT_CHANGED_THREAD_PARTITION_ID,
    OT_CHANGED_THREAD_RLOC_ADDED, OT_CHANGED_THREAD_RLOC_REMOVED, OT_CHANGED_THREAD_ROLE,
    OT_NETWORK_NAME_MAX_SIZE,
};

use crate::{
    radio::{RCV_FRAME, RCV_FRAME_PSDU},
    timer::current_millis,
};

static RADIO: Mutex<RefCell<Option<&'static mut Ieee802154>>> = Mutex::new(RefCell::new(None));

static NETWORK_SETTINGS: Mutex<RefCell<Option<NetworkSettings>>> = Mutex::new(RefCell::new(None));

static RADIO_SETTINGS: Mutex<RefCell<Option<Config>>> = Mutex::new(RefCell::new(None));

static CHANGE_CALLBACK: Mutex<RefCell<Option<&'static mut (dyn FnMut(ChangedFlags) + Send)>>> =
    Mutex::new(RefCell::new(None));

pub(crate) static TIMER: Mutex<RefCell<Option<Alarm<Target, esp_hal::Blocking, 0>>>> =
    Mutex::new(RefCell::new(None));

pub(crate) static TIMER_CALLBACK_SHOULD_RUN: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));

pub(crate) static mut CURRENT_INSTANCE: usize = 0;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[doc(hidden)]
#[macro_export]
macro_rules! checked {
    ($value:expr) => {
        if $value != 0 {
            Err(crate::Error::InternalError($value))
        } else {
            core::result::Result::<(), crate::Error>::Ok(())
        }
    };
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Error {
    InternalError(u32),
}

// TODO: Make these generated as part of bindgen task?
bitflags! {
    /// Specific state/configuration that has changed
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ChangedFlags: u32 {
        /// IPv6 address was added
        const Ipv6AddressAdded = OT_CHANGED_IP6_ADDRESS_ADDED;
        /// IPv6 address was removed
        const Ipv6AddressRemoved = OT_CHANGED_IP6_ADDRESS_REMOVED;
        /// Role (disabled, detached, child, router, leader) changed
        const ThreadRoleChanged = OT_CHANGED_THREAD_ROLE;
        /// The link-local address changed
        const ThreadLlAddressChanged = OT_CHANGED_THREAD_LL_ADDR;
        /// The mesh-local address changed
        const ThreadMeshLocalAddressChanged = OT_CHANGED_THREAD_ML_ADDR;
        ///  RLOC was added
        const ThreadRlocAdded = OT_CHANGED_THREAD_RLOC_ADDED;
        /// RLOC was removed
        const ThreadRlocRemoved = OT_CHANGED_THREAD_RLOC_REMOVED;
        /// Partition ID changed
        const ThreadPartitionIdChanged = OT_CHANGED_THREAD_PARTITION_ID;
        /// Thread Key Sequence changed
        const ThreadKeySequenceChanged = OT_CHANGED_THREAD_KEY_SEQUENCE_COUNTER;
        /// Thread Network Data changed
        const ThreadNetworkDataChanged = OT_CHANGED_THREAD_NETDATA;
        /// Child was added
        const ThreadChildAdded = OT_CHANGED_THREAD_CHILD_ADDED;
        /// Child was removed
        const ThreadChildRemoved = OT_CHANGED_THREAD_CHILD_REMOVED;
        /// Subscribed to a IPv6 multicast address
        const Ipv6MulticastSubscribed = OT_CHANGED_IP6_MULTICAST_SUBSCRIBED;
        /// Unsubscribed from a IPv6 multicast address
        const Ipv6MulticastUnsubscribed = OT_CHANGED_IP6_MULTICAST_UNSUBSCRIBED;
        /// Thread network channel changed
        const ThreadNetworkChannelChanged = OT_CHANGED_THREAD_CHANNEL;
        /// Thread network PAN Id changed
        const ThreadPanIdChanged = OT_CHANGED_THREAD_PANID;
        /// Thread network name changed
        const ThreadNetworkNameChanged = OT_CHANGED_THREAD_NETWORK_NAME;
        /// Thread network extended PAN ID changed
        const ThreadExtendedPanIdChanged = OT_CHANGED_THREAD_EXT_PANID;
        /// Network key changed
        const ThreadNetworkKeyChanged = OT_CHANGED_NETWORK_KEY;
        /// PSKc changed
        const ThreadPskcChanged = OT_CHANGED_PSKC;
        /// Security Policy changed
        const ThreadSecurityPolicyChanged = OT_CHANGED_SECURITY_POLICY;
        /// Channel Manager new pending Thread channel changed
        const ChannelManagerNewChannelChanged = OT_CHANGED_CHANNEL_MANAGER_NEW_CHANNEL;
        /// Supported channel mask changed
        const SupportedChannelMaskChanged = OT_CHANGED_SUPPORTED_CHANNEL_MASK;
        /// Commissioner state changed
        const CommissionerStateChanged = OT_CHANGED_COMMISSIONER_STATE;
        /// Thread network interface state changed
        const ThreadNetworkInterfaceStateChanged = OT_CHANGED_THREAD_NETIF_STATE;
        /// Backbone Router state changed
        const ThreadBackboneRouterStateChanged = OT_CHANGED_THREAD_BACKBONE_ROUTER_STATE;
        /// Local Backbone Router configuration changed
        const ThreadBackboneRouterLocalChanged = OT_CHANGED_THREAD_BACKBONE_ROUTER_LOCAL;
        /// Joiner state changed
        const JoinerStateChanged = OT_CHANGED_JOINER_STATE;
        /// Active Operational Dataset changed
        const ActiveDatasetChanged = OT_CHANGED_ACTIVE_DATASET;
        /// Pending Operational Dataset changed
        const PendingDatasetChanged = OT_CHANGED_PENDING_DATASET;
        /// State of the Nat64 Translator changed
        const Nat64TranslatorStateChanged= OT_CHANGED_NAT64_TRANSLATOR_STATE;
        /// Parent link quality changed
        const ParentLinkQualityChanged = OT_CHANGED_PARENT_LINK_QUALITY;
    }
}

/// IPv6 network interface unicast address
#[derive(Debug, Clone, Copy)]
pub struct NetworkInterfaceUnicastAddress {
    /// The IPv6 unicast address
    pub address: no_std_net::Ipv6Addr,
    /// The Prefix length (in bits)
    pub prefix: u8,
    /// The IPv6 address origin
    pub origin: u8,
}

/// Thread Dataset timestamp
#[derive(Debug, Clone, Copy)]
pub struct ThreadTimestamp {
    pub seconds: u64,
    pub ticks: u16,
    pub authoritative: bool,
}

/// Security Policy
#[derive(Debug, Clone, Default)]
pub struct SecurityPolicy {
    /// The value for thrKeyRotation in units of hours.
    pub rotation_time: u16,
    /// Autonomous Enrollment is enabled.
    pub autonomous_enrollment_enabled: bool,
    /// Commercial Commissioning is enabled.
    pub commercial_commissioning_enabled: bool,
    /// External Commissioner authentication is allowed.
    pub external_commissioning_enabled: bool,
    /// Native Commissioning using PSKc is allowed.
    pub native_commissioning_enabled: bool,
    /// Network Key Provisioning is enabled.
    pub network_key_provisioning_enabled: bool,
    /// Non-CCM Routers enabled.
    pub non_ccm_routers_enabled: bool,
    /// Obtaining the Network Key for out-of-band commissioning is enabled.
    pub obtain_network_key_enabled: bool,
    /// Thread 1.0/1.1.x Routers are enabled.
    pub routers_enabled: bool,
    /// ToBLE link is enabled.
    pub toble_link_enabled: bool,
    /// Version-threshold for Routing.
    pub version_threshold_for_routing: u8,
}

/// Active or Pending Operational Dataset
#[derive(Debug, Clone, Default)]
pub struct OperationalDataset {
    /// Active Timestamp
    pub active_timestamp: Option<ThreadTimestamp>,
    /// Pending Timestamp
    pub pending_timestamp: Option<ThreadTimestamp>,
    /// Network Key
    pub network_key: Option<[u8; 16]>,
    /// Network name
    pub network_name: Option<heapless::String<{ OT_NETWORK_NAME_MAX_SIZE as usize }>>,
    /// Extended PAN ID
    pub extended_pan_id: Option<[u8; 8]>,
    /// Mesh Local Prefix
    pub mesh_local_prefix: Option<[u8; 8]>,
    /// Delay Timer
    pub delay: Option<u32>,
    /// PAN ID
    pub pan_id: Option<u16>,
    /// Channel
    pub channel: Option<u16>,
    /// PSKc
    pub pskc: Option<[u8; 16]>,
    /// Security Policy.
    pub security_policy: Option<SecurityPolicy>,
    /// Channel Mask
    pub channel_mask: Option<u32>,
}

#[derive(Debug, Clone, Copy, Default)]
struct NetworkSettings {
    promiscuous: bool,
    ext_address: u64,
    short_address: u16,
    pan_id: u16,
    channel: u8,
}

/// Instance of OpenThread
#[non_exhaustive]
pub struct OpenThread<'a> {
    _phantom: PhantomData<&'a ()>,
    // pub for now
    pub instance: *mut otInstance,
}

impl<'a> OpenThread<'a> {
    pub fn new(
        radio: &'a mut Ieee802154,
        timer: Alarm<Target, esp_hal::Blocking, 0>,
        rng: esp_hal::rng::Rng,
    ) -> Self {
        timer::install_isr(timer);
        entropy::init_rng(rng);

        radio.set_tx_done_callback_fn(radio::trigger_tx_done);

        critical_section::with(|cs| {
            RADIO
                .borrow_ref_mut(cs)
                .replace(unsafe { core::mem::transmute(radio) });
        });

        let instance = unsafe { otInstanceInitSingle() };
        log::debug!("otInstanceInitSingle done, instance = {:p}", instance);

        let res = unsafe {
            otSetStateChangedCallback(instance, Some(change_callback), core::ptr::null_mut())
        };
        log::debug!("otSetStateChangedCallback {res}");

        Self {
            _phantom: PhantomData,
            instance,
        }
    }

    pub fn set_radio_config(&mut self, config: Config) -> Result<()> {
        critical_section::with(|cs| {
            let mut radio = RADIO.borrow_ref_mut(cs);
            let radio = radio.borrow_mut();

            if let Some(radio) = radio.as_mut() {
                radio.set_config(config)
            }
        });
        set_radio_config(config);

        Ok(())
    }

    /// Sets the Active Operational Dataset
    pub fn set_active_dataset(&mut self, dataset: OperationalDataset) -> Result<()> {
        let mut raw_dataset = otOperationalDataset {
            mActiveTimestamp: otTimestamp {
                mSeconds: 0,
                mTicks: 0,
                mAuthoritative: false,
            },
            mPendingTimestamp: otTimestamp {
                mSeconds: 0,
                mTicks: 0,
                mAuthoritative: false,
            },
            mNetworkKey: otNetworkKey { m8: [0u8; 16] },
            mNetworkName: otNetworkName { m8: [0i8; 17] },
            mExtendedPanId: otExtendedPanId { m8: [0u8; 8] },
            mMeshLocalPrefix: otMeshLocalPrefix { m8: [0u8; 8] },
            mDelay: 0,
            mPanId: 0,
            mChannel: 0,
            mPskc: otPskc { m8: [0u8; 16] },
            mSecurityPolicy: otSecurityPolicy {
                mRotationTime: 0,
                _bitfield_align_1: [0u8; 0],
                _bitfield_1: otSecurityPolicy::new_bitfield_1(
                    false, false, false, false, false, false, false, false, false, 0,
                ),
            },
            mChannelMask: 0,
            mComponents: otOperationalDatasetComponents {
                mIsActiveTimestampPresent: true,
                mIsPendingTimestampPresent: false,
                mIsNetworkKeyPresent: true,
                mIsNetworkNamePresent: true,
                mIsExtendedPanIdPresent: true,
                mIsMeshLocalPrefixPresent: false,
                mIsDelayPresent: false,
                mIsPanIdPresent: true,
                mIsChannelPresent: true,
                mIsPskcPresent: false,
                mIsSecurityPolicyPresent: false,
                mIsChannelMaskPresent: false,
            },
        };

        let mut active_timestamp_present = false;
        let mut pending_timestamp_present = false;
        let mut network_key_present = false;
        let mut network_name_present = false;
        let mut extended_pan_present = false;
        let mut mesh_local_prefix_present = false;
        let mut delay_present = false;
        let mut pan_id_present = false;
        let mut channel_present = false;
        let mut pskc_present = false;
        let mut security_policy_present = false;
        let mut channel_mask_present = false;

        if let Some(active_timestamp) = dataset.active_timestamp {
            raw_dataset.mActiveTimestamp = otTimestamp {
                mSeconds: active_timestamp.seconds,
                mTicks: active_timestamp.ticks,
                mAuthoritative: active_timestamp.authoritative,
            };
            active_timestamp_present = true;
        }

        if let Some(pending_timestamp) = dataset.pending_timestamp {
            raw_dataset.mActiveTimestamp = otTimestamp {
                mSeconds: pending_timestamp.seconds,
                mTicks: pending_timestamp.ticks,
                mAuthoritative: pending_timestamp.authoritative,
            };
            pending_timestamp_present = true;
        }

        if let Some(network_key) = dataset.network_key {
            raw_dataset.mNetworkKey = otNetworkKey { m8: network_key };
            network_key_present = true;
        }

        if let Some(network_name) = dataset.network_name {
            let mut raw = [0i8; 17];
            raw[..network_name.len()]
                .copy_from_slice(unsafe { core::mem::transmute(network_name.as_bytes()) });
            raw_dataset.mNetworkName = otNetworkName { m8: raw };
            network_name_present = true;
        }

        if let Some(extended_pan_id) = dataset.extended_pan_id {
            raw_dataset.mExtendedPanId = otExtendedPanId {
                m8: extended_pan_id,
            };
            extended_pan_present = true;
        }

        if let Some(mesh_local_prefix) = dataset.mesh_local_prefix {
            raw_dataset.mMeshLocalPrefix = otMeshLocalPrefix {
                m8: mesh_local_prefix,
            };
            mesh_local_prefix_present = true;
        }

        if let Some(delay) = dataset.delay {
            raw_dataset.mDelay = delay;
            delay_present = true;
        }

        if let Some(pan_id) = dataset.pan_id {
            raw_dataset.mPanId = pan_id;
            pan_id_present = true;
            let settings: NetworkSettings = get_settings();
            set_settings(NetworkSettings { pan_id, ..settings });
        }

        if let Some(channel) = dataset.channel {
            raw_dataset.mChannel = channel;
            channel_present = true;
            let settings: NetworkSettings = get_settings();
            set_settings(NetworkSettings {
                channel: channel as u8,
                ..settings
            });
        }

        if let Some(pskc) = dataset.pskc {
            raw_dataset.mPskc = otPskc { m8: pskc };
            pskc_present = true;
        }

        if let Some(security_policy) = dataset.security_policy {
            raw_dataset.mSecurityPolicy = otSecurityPolicy {
                mRotationTime: security_policy.rotation_time,
                _bitfield_align_1: [0u8; 0],
                _bitfield_1: otSecurityPolicy::new_bitfield_1(
                    security_policy.obtain_network_key_enabled,
                    security_policy.native_commissioning_enabled,
                    security_policy.routers_enabled,
                    security_policy.external_commissioning_enabled,
                    security_policy.commercial_commissioning_enabled,
                    security_policy.autonomous_enrollment_enabled,
                    security_policy.network_key_provisioning_enabled,
                    security_policy.toble_link_enabled,
                    security_policy.non_ccm_routers_enabled,
                    security_policy.version_threshold_for_routing,
                ),
            };
            security_policy_present = true;
        }

        if let Some(channel_mask) = dataset.channel_mask {
            raw_dataset.mChannelMask = channel_mask;
            channel_mask_present = true;
        }

        raw_dataset.mComponents = otOperationalDatasetComponents {
            mIsActiveTimestampPresent: active_timestamp_present,
            mIsPendingTimestampPresent: pending_timestamp_present,
            mIsNetworkKeyPresent: network_key_present,
            mIsNetworkNamePresent: network_name_present,
            mIsExtendedPanIdPresent: extended_pan_present,
            mIsMeshLocalPrefixPresent: mesh_local_prefix_present,
            mIsDelayPresent: delay_present,
            mIsPanIdPresent: pan_id_present,
            mIsChannelPresent: channel_present,
            mIsPskcPresent: pskc_present,
            mIsSecurityPolicyPresent: security_policy_present,
            mIsChannelMaskPresent: channel_mask_present,
        };

        checked!(unsafe { otDatasetSetActive(self.instance, &raw_dataset) })
    }

    /// Set the change callback
    pub fn set_change_callback(
        &mut self,
        callback: Option<&'a mut (dyn FnMut(ChangedFlags) + Send)>,
    ) {
        critical_section::with(|cs| {
            let mut change_callback = CHANGE_CALLBACK.borrow_ref_mut(cs);
            *change_callback = unsafe { core::mem::transmute(callback) };
        });
    }

    /// Brings the IPv6 interface up or down.
    pub fn ipv6_set_enabled(&mut self, enabled: bool) -> Result<()> {
        checked!(unsafe { otIp6SetEnabled(self.instance, enabled) })
    }

    /// This function starts Thread protocol operation.
    ///
    /// The interface must be up when calling this function.
    pub fn thread_set_enabled(&mut self, enabled: bool) -> Result<()> {
        checked!(unsafe { otThreadSetEnabled(self.instance, enabled) })
    }

    /// This function returns the device's EUI
    ///
    /// EUI is read from efuse
    pub fn get_eui(&self, out: &mut [u8]) {
        unsafe { otPlatRadioGetIeeeEui64(self.instance, out.as_mut_ptr()) }
    }

    /// Gets the list of IPv6 addresses assigned to the Thread interface.
    pub fn ipv6_get_unicast_addresses<const N: usize>(
        &self,
    ) -> heapless::Vec<NetworkInterfaceUnicastAddress, N> {
        let mut result = heapless::Vec::new();
        let mut addr = unsafe { otIp6GetUnicastAddresses(self.instance) };
        loop {
            let a = unsafe { &(*(addr)) };

            let octets = unsafe { a.mAddress.mFields.m16 };

            if result
                .push(NetworkInterfaceUnicastAddress {
                    address: no_std_net::Ipv6Addr::new(
                        octets[0].to_be(),
                        octets[1].to_be(),
                        octets[2].to_be(),
                        octets[3].to_be(),
                        octets[4].to_be(),
                        octets[5].to_be(),
                        octets[6].to_be(),
                        octets[7].to_be(),
                    ),
                    prefix: a.mPrefixLength,
                    origin: a.mAddressOrigin,
                })
                .is_err()
            {
                break;
            }

            if a.mNext.is_null() {
                break;
            }

            addr = a.mNext;
        }

        result
    }

    /// Creates a new UDP socket
    pub fn get_udp_socket<'s, const BUFFER_SIZE: usize>(
        &'s self,
    ) -> Result<UdpSocket<'s, 'a, BUFFER_SIZE>, Error>
    where
        'a: 's,
    {
        crate::net::udp::UdpSocket::get_udp_socket(self)
    }

    /// Run tasks
    ///
    /// Make sure to periodically call this function.
    pub fn run_tasklets(&self) {
        unsafe {
            if otTaskletsArePending(self.instance) {
                otTaskletsProcess(self.instance);
            }
        }
    }

    /// Run due timers, get and forward received messages
    ///
    /// Make sure to periodically call this function.
    pub fn process(&self) {
        crate::timer::run_if_due();

        while let Some(raw) = with_radio(|radio| radio.get_raw_received()).unwrap() {
            let rssi = raw.data[raw.data[0] as usize - 1] as i8;
            unsafe {
                let len = raw.data[0];
                log::debug!("RCV {:02x?}", &raw.data[1..][..len as usize]);

                RCV_FRAME_PSDU[..len as usize].copy_from_slice(&raw.data[1..][..len as usize]);
                RCV_FRAME.mLength = len as u16;
                RCV_FRAME.mRadioType = 1; // ????
                RCV_FRAME.mChannel = raw.channel;
                RCV_FRAME.mInfo.mRxInfo.mRssi = rssi;
                RCV_FRAME.mInfo.mRxInfo.mLqi = rssi_to_lqi(rssi);
                RCV_FRAME.mInfo.mRxInfo.mTimestamp = current_millis() * 1000;
                otPlatRadioReceiveDone(
                    self.instance,
                    addr_of_mut!(RCV_FRAME),
                    otError_OT_ERROR_NONE,
                );
            }
        }
    }

    pub fn set_child_timeout(&mut self, timeout: u32) -> Result<()> {
        unsafe { otThreadSetChildTimeout(self.instance, timeout) };
        Ok(())
    }

    /// When device is MTD, device type should be false, full network data should be false,
    /// and rx on when idle should be set accordingly (likely false depending on other config)
    pub fn set_link_mode(
        &mut self,
        rx_on_when_idle: bool,
        device_type: bool,
        network_data: bool,
    ) -> Result<()> {
        let mut link_mode = self.get_link_mode();
        link_mode.set_mRxOnWhenIdle(rx_on_when_idle);
        link_mode.set_mDeviceType(device_type);
        link_mode.set_mNetworkData(network_data);

        unsafe { otThreadSetLinkMode(self.instance, link_mode) };
        Ok(())
    }


    pub fn get_link_mode(
        &self,
    ) -> otLinkModeConfig {
        unsafe { otThreadGetLinkMode(self.instance) }
    }

    pub fn get_device_role(&self) -> ThreadDeviceRole {
        let role = unsafe { otThreadGetDeviceRole(self.instance) };
        role.into()
    }
}

#[derive(Debug)]
pub enum ThreadDeviceRole {
    Disabled,
    Detached,
    Child,
    Router,
    Leader,
    Unknown,
}

#[allow(non_upper_case_globals)]
impl From<otDeviceRole> for ThreadDeviceRole {
    fn from(role: otDeviceRole) -> Self {
        match role {
            otDeviceRole_OT_DEVICE_ROLE_DISABLED => ThreadDeviceRole::Disabled,
            otDeviceRole_OT_DEVICE_ROLE_DETACHED => ThreadDeviceRole::Detached,
            otDeviceRole_OT_DEVICE_ROLE_CHILD => ThreadDeviceRole::Child,
            otDeviceRole_OT_DEVICE_ROLE_ROUTER => ThreadDeviceRole::Router,
            otDeviceRole_OT_DEVICE_ROLE_LEADER => ThreadDeviceRole::Leader,
            _ => ThreadDeviceRole::Unknown,
        }
    }
}

impl core::fmt::Display for ThreadDeviceRole {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ThreadDeviceRole::Disabled => write!(f, "Disabled"),
            ThreadDeviceRole::Detached => write!(f, "Detached"),
            ThreadDeviceRole::Child => write!(f, "Child"),
            ThreadDeviceRole::Router => write!(f, "Router"),
            ThreadDeviceRole::Leader => write!(f, "Leader"),
            ThreadDeviceRole::Unknown => write!(f, "Unknown"),
        }
    }
}

impl<'a> Drop for OpenThread<'a> {
    fn drop(&mut self) {
        critical_section::with(|cs| {
            RADIO.borrow_ref_mut(cs).take();
            NETWORK_SETTINGS.borrow_ref_mut(cs).take();
            CHANGE_CALLBACK.borrow_ref_mut(cs).take();
        });
    }
}

unsafe extern "C" fn change_callback(
    flags: otChangedFlags,
    _context: *mut esp_openthread_sys::c_types::c_void,
) {
    log::debug!("change_callback otChangedFlags={:32b}", flags);
    critical_section::with(|cs| {
        let mut change_callback = CHANGE_CALLBACK.borrow_ref_mut(cs);
        let callback = change_callback.as_mut();

        if let Some(callback) = callback {
            if ChangedFlags::from_bits(flags).is_none() {
                log::warn!(
                    "change_callback otChangedFlags= {:?} would be None as flags",
                    flags
                );
            } else {
                callback(ChangedFlags::from_bits(flags).unwrap());
            }
        }
    });
}

fn with_radio<F, T>(f: F) -> Option<T>
where
    F: FnOnce(&mut Ieee802154) -> T,
{
    critical_section::with(|cs| {
        let mut radio = RADIO.borrow_ref_mut(cs);
        let radio = radio.borrow_mut();

        if let Some(radio) = radio.as_mut() {
            Some(f(radio))
        } else {
            None
        }
    })
}

fn get_settings() -> NetworkSettings {
    critical_section::with(|cs| {
        let mut settings = NETWORK_SETTINGS.borrow_ref_mut(cs);
        let settings = settings.borrow_mut();

        if let Some(settings) = settings.as_mut() {
            settings.clone()
        } else {
            log::warn!("Generating default settings");
            NetworkSettings::default()
        }
    })
}

fn set_settings(settings: NetworkSettings) {
    critical_section::with(|cs| {
        log::debug!(
            "Setting settings to {:?}\nwere {:?}",
            settings,
            NETWORK_SETTINGS.borrow_ref(cs)
        );
        NETWORK_SETTINGS
            .borrow_ref_mut(cs)
            .borrow_mut()
            .replace(settings);
    });
}

fn get_radio_config() -> Config {
    critical_section::with(|cs| {
        let mut settings = RADIO_SETTINGS.borrow_ref_mut(cs);
        let settings = settings.borrow_mut();

        if let Some(settings) = settings.as_mut() {
            settings.clone()
        } else {
            log::warn!("Generating default radio settings");
            Config::default()
        }
    })
}

fn set_radio_config(settings: Config) {
    critical_section::with(|cs| {
        log::debug!(
            "RADIO settings to {:?}\nwere {:?}",
            settings,
            RADIO_SETTINGS.borrow_ref(cs)
        );
        RADIO_SETTINGS
            .borrow_ref_mut(cs)
            .borrow_mut()
            .replace(settings);
    });
}
