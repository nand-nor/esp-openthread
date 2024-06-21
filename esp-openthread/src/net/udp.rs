use core::{marker::PhantomPinned, pin::Pin};

use crate::{checked, Error, OpenThread, Result};
pub use esp_openthread_sys as sys;
use no_std_net::Ipv6Addr;
use sys::{
    bindings::{
        __BindgenBitfieldUnit, otError_OT_ERROR_NONE, otIp6Address, otIp6Address__bindgen_ty_1,
        otMessage, otMessageAppend, otMessageFree, otMessageGetLength, otMessageInfo,
        otMessageRead, otNetifIdentifier_OT_NETIF_THREAD, otSockAddr, otUdpBind, otUdpClose,
        otUdpNewMessage, otUdpOpen, otUdpSend, otUdpSocket,
    },
    c_types::c_void,
};

/// A UdpSocket
///
/// To call functions on it you have to pin it.
/// ```no_run
/// let mut socket = openthread.get_udp_socket::<512>().unwrap();
/// let mut socket = pin!(socket);
/// socket.bind(1212).unwrap();
/// ```
pub struct UdpSocket<'s, 'n: 's, const BUFFER_SIZE: usize> {
    ot_socket: otUdpSocket,
    ot: &'s OpenThread<'n>,
    receive_len: usize,
    receive_from: [u8; 16],
    receive_port: u16,
    max: usize,
    _pinned: PhantomPinned,
    // must be last because the callback doesn't know about the actual const generic parameter
    receive_buffer: [u8; BUFFER_SIZE],
}

impl<'s, 'n: 's, const BUFFER_SIZE: usize> UdpSocket<'s, 'n, BUFFER_SIZE> {
    /// Creates a new UDP socket
    pub fn get_udp_socket(
        instance: &'s OpenThread<'n>,
    ) -> Result<UdpSocket<'s, 'n, { BUFFER_SIZE }>, Error>
    where
        'n: 's,
    {
        let ot_socket = otUdpSocket {
            mSockName: otSockAddr {
                mAddress: otIp6Address {
                    mFields: otIp6Address__bindgen_ty_1 { m32: [0, 0, 0, 0] },
                },
                mPort: 0,
            },
            mPeerName: otSockAddr {
                mAddress: otIp6Address {
                    mFields: otIp6Address__bindgen_ty_1 { m32: [0, 0, 0, 0] },
                },
                mPort: 0,
            },
            mHandler: Some(udp_receive_handler),
            mContext: core::ptr::null_mut(),
            mHandle: core::ptr::null_mut(),
            mNext: core::ptr::null_mut(),
        };

        Ok(Self {
            ot_socket,
            ot: instance,
            receive_len: 0,
            receive_from: [0u8; 16],
            receive_port: 0,
            max: BUFFER_SIZE,
            _pinned: PhantomPinned::default(),
            receive_buffer: [0u8; BUFFER_SIZE],
        })
    }

    /// Open and bind a UDP/IPv6 socket
    pub fn bind(self: &mut Pin<&mut Self>, port: u16) -> Result<()> {
        let mut sock_addr = otSockAddr {
            mAddress: otIp6Address {
                mFields: otIp6Address__bindgen_ty_1 { m32: [0, 0, 0, 0] },
            },
            mPort: 0,
        };
        sock_addr.mPort = port;

        unsafe {
            checked!(otUdpOpen(
                self.ot.instance,
                &self.ot_socket as *const _ as *mut otUdpSocket,
                Some(udp_receive_handler),
                self.as_mut().get_unchecked_mut() as *mut _ as *mut crate::sys::c_types::c_void,
            ))?;
        }

        unsafe {
            checked!(otUdpBind(
                self.ot.instance,
                &self.ot_socket as *const _ as *mut otUdpSocket,
                &mut sock_addr,
                otNetifIdentifier_OT_NETIF_THREAD,
            ))?;
        }

        Ok(())
    }

    /// Open a UDP/IPv6 socket
    pub fn open(self: &mut Pin<&mut Self>, port: u16) -> Result<()> {
        let mut sock_addr = otSockAddr {
            mAddress: otIp6Address {
                mFields: otIp6Address__bindgen_ty_1 { m32: [0, 0, 0, 0] },
            },
            mPort: 0,
        };
        sock_addr.mPort = port;

        unsafe {
            checked!(otUdpOpen(
                self.ot.instance,
                &self.ot_socket as *const _ as *mut otUdpSocket,
                Some(udp_receive_handler),
                self.as_mut().get_unchecked_mut() as *mut _ as *mut crate::sys::c_types::c_void,
            ))?;
        }
        Ok(())
    }

    /// Get latest data received on this socket
    pub fn receive(
        self: &mut Pin<&mut Self>,
        data: &mut [u8],
    ) -> Result<(usize, Ipv6Addr, u16), Error> {
        critical_section::with(|_| {
            let len = self.receive_len as usize;
            if len == 0 {
                Ok((0, Ipv6Addr::UNSPECIFIED, 0))
            } else {
                unsafe { self.as_mut().get_unchecked_mut() }.receive_len = 0;
                data[..len].copy_from_slice(&self.receive_buffer[..len]);
                let ip = Ipv6Addr::from(self.receive_from);
                Ok((len, ip, self.receive_port))
            }
        })
    }

    /// Send data to the given peer
    pub fn send(self: &mut Pin<&mut Self>, dst: Ipv6Addr, port: u16, data: &[u8]) -> Result<()> {
        let mut message_info = otMessageInfo {
            mSockAddr: otIp6Address {
                mFields: otIp6Address__bindgen_ty_1 { m32: [0, 0, 0, 0] },
            },
            mPeerAddr: otIp6Address {
                mFields: otIp6Address__bindgen_ty_1 { m32: [0, 0, 0, 0] },
            },
            mSockPort: 0,
            mPeerPort: 0,
            mHopLimit: 0,
            _bitfield_align_1: [0u8; 0],
            _bitfield_1: __BindgenBitfieldUnit::new([0u8; 1]),
        };
        message_info.mPeerAddr.mFields.m8 = dst.octets();
        message_info.mPeerPort = port;

        let message = unsafe { otUdpNewMessage(self.ot.instance, core::ptr::null()) };
        if message.is_null() {
            return Err(Error::InternalError(0));
        }

        unsafe {
            checked!(otMessageAppend(
                message,
                data.as_ptr() as *const c_void,
                data.len() as u16
            ))?;
        }

        unsafe {
            let err = otUdpSend(
                self.ot.instance,
                &self.ot_socket as *const _ as *mut otUdpSocket,
                message,
                &mut message_info,
            );

            if err != otError_OT_ERROR_NONE && !message.is_null() {
                otMessageFree(message);
                return Err(Error::InternalError(err));
            }
        }

        Ok(())
    }

    /// Close a UDP/IPv6 socket
    pub fn close(self: &mut Pin<&mut Self>) -> Result<()> {
        unsafe {
            checked!(otUdpClose(
                self.ot.instance,
                &self.ot_socket as *const _ as *mut otUdpSocket,
            ))?;
        }

        Ok(())
    }

    fn close_internal(&mut self) -> Result<()> {
        unsafe {
            checked!(otUdpClose(
                self.ot.instance,
                &self.ot_socket as *const _ as *mut otUdpSocket,
            ))?;
        }

        Ok(())
    }
}

impl<'s, 'n: 's, const BUFFER_SIZE: usize> Drop for UdpSocket<'s, 'n, BUFFER_SIZE> {
    fn drop(&mut self) {
        self.close_internal().ok();
    }
}

pub unsafe extern "C" fn udp_receive_handler(
    context: *mut crate::sys::c_types::c_void,
    message: *mut otMessage,
    message_info: *const otMessageInfo,
) {
    let socket = context as *mut UdpSocket<1024>;
    let len = u16::min((*socket).max as u16, otMessageGetLength(message));

    critical_section::with(|_| {
        otMessageRead(
            message,
            0,
            &mut (*socket).receive_buffer as *mut _ as *mut crate::sys::c_types::c_void,
            len,
        );
        (*socket).receive_port = (*message_info).mPeerPort;
        (*socket).receive_from = (*message_info).mPeerAddr.mFields.m8;
        (*socket).receive_len = len as usize;
    });
}
