//! Example showing use of SRP client APIs

#![no_std]
#![no_main]

use core::{arch::asm, borrow::BorrowMut, cell::RefCell, pin::pin};

use critical_section::Mutex;
use esp_backtrace as _;
use esp_hal::{
    prelude::*,
    rng::Rng,
    timer::systimer::{Alarm, FrozenUnit, SpecificUnit, SystemTimer},
};
use esp_ieee802154::Ieee802154;
use esp_openthread::{
    sys::bindings::{otIp6Address, otSockAddr, otIp6Address__bindgen_ty_1}, ChangedFlags, NetworkInterfaceUnicastAddress, OperationalDataset, ThreadTimestamp
};
use esp_println::println;
use static_cell::StaticCell;

// Host names and service names must be unique and are
// accepted by SRP server on first come first serve basis.
// Note that in the code below the names are modified,
// at runtime, to avoid collisions
static HOSTNAME: Mutex<RefCell<&'static str>> = Mutex::new(RefCell::new(BASE_HOSTNAME));
static SERVICENAME: Mutex<RefCell<&'static str>> = Mutex::new(RefCell::new(BASE_SERVICENAME));
static INSTANCENAME: Mutex<RefCell<&'static str>> = Mutex::new(RefCell::new(BASE_INSTANCENAME));
static DNSTXT: Mutex<RefCell<&'static str>> = Mutex::new(RefCell::new(BASE_DNSTXT));
static SUBTYPES: Mutex<RefCell<&'static [&'static str]>> = Mutex::new(RefCell::new(&[]));

const BASE_HOSTNAME: &str = "-ot-esp32\0";
const BASE_SERVICENAME: &str = "-ot-service\0";
const BASE_INSTANCENAME: &str = "_otpps._udp\0";
const BASE_DNSTXT: &str = "key=30\0";
const BASE_SUBTYPE: &str = ",_tlp\0";

const BOUND_PORT: u16 = 1212;

extern crate alloc;

use alloc::string::{ToString, String};

#[entry] 
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    esp_alloc::heap_allocator!(32 * 1024);

    let mut peripherals = esp_hal::init(esp_hal::Config::default());

    println!("Initializing");

    let radio = peripherals.IEEE802154;
    let mut ieee802154 = Ieee802154::new(radio, &mut peripherals.RADIO_CLK);

    // init timer for otPlatAlarm
    let systimer = SystemTimer::new(peripherals.SYSTIMER);
    static UNIT0: StaticCell<SpecificUnit<'static, 0>> = StaticCell::new();
    let unit0 = UNIT0.init(systimer.unit0);
    let frozen_unit = FrozenUnit::new(unit0);
    let alarm = Alarm::new(systimer.comparator0, &frozen_unit);

    let mut openthread =
        esp_openthread::OpenThread::new(&mut ieee802154, alarm, Rng::new(peripherals.RNG));

    let changed = Mutex::new(RefCell::new((false, ChangedFlags::Ipv6AddressAdded)));
    let mut callback = |flags| {
        println!("{:?}", flags);
        critical_section::with(|cs| *changed.borrow_ref_mut(cs) = (true, flags));
    };

    openthread.set_change_callback(Some(&mut callback));

    let dataset = OperationalDataset {
        active_timestamp: Some(ThreadTimestamp {
            seconds: 1,
            ticks: 0,
            authoritative: false,
        }),
        network_key: Some([
            0xfe, 0x04, 0x58, 0xf7, 0xdb, 0x96, 0x35, 0x4e, 0xaa, 0x60, 0x41, 0xb8, 0x80, 0xea,
            0x9c, 0x0f,
        ]),
        network_name: Some("OpenThread-58d1".try_into().unwrap()),
        extended_pan_id: Some([0x3a, 0x90, 0xe3, 0xa3, 0x19, 0xa9, 0x04, 0x94]),
        pan_id: Some(0x58d1),
        channel: Some(25),
        channel_mask: Some(0x07fff800),
        ..OperationalDataset::default()
    };
    println!("dataset : {:?}", dataset);

    openthread.set_active_dataset(dataset).unwrap();

    let srp_changed = Mutex::new(RefCell::new((0, 0, 0, 0,)));
    let mut srp_callback = |error, a, b, c| {
        println!("SRP error callback: {:?}", error);
        critical_section::with(|cs| *srp_changed.borrow_ref_mut(cs) = (error, a, b, c));
    };

    openthread.set_srp_state_callback(Some(&mut srp_callback));

    let rand = esp_openthread::get_random_u32().to_string();

    // Add some random bytes so host name and service name are "unique"
    // every time this code runs (to avoid SRP collisions)
    let mut base_host: String = rand.clone();
    base_host.push_str(BASE_HOSTNAME);

    let mut base_srvc: String = rand;
    base_srvc.push_str(BASE_SERVICENAME);

    critical_section::with(|cs| {
        let mut host = HOSTNAME.borrow_ref_mut(cs);
        let host = (&mut *host).borrow_mut();
        *host = unsafe { core::mem::transmute(base_host.as_str() ) };
        let mut srvc = SERVICENAME.borrow_ref_mut(cs);
        let srvc = (&mut *srvc).borrow_mut();
        *srvc = unsafe { core::mem::transmute(base_srvc.as_str() ) };
    });

    openthread.ipv6_set_enabled(true).unwrap();

    openthread.thread_set_enabled(true).unwrap();

    let addrs: heapless::Vec<NetworkInterfaceUnicastAddress, 5> =
        openthread.ipv6_get_unicast_addresses();

    print_all_addresses(addrs);
    
    // stop the client before registering if it is running
    // Note: the esp-openthread lib currently is built with autostart enabled
    if openthread.is_srp_client_running(){
        openthread.stop_srp_client().ok();
    }

    let mut register = false;

    critical_section::with(|cs| {
        let mut host = HOSTNAME.borrow_ref_mut(cs);
        let host = host.borrow_mut();

        if let Err(e) = openthread.setup_srp_client_set_hostname((*host).as_ref()) {
            log::error!("Error enabling srp client {e:?}");
        }

    });
    
    if let Err(e) = openthread.setup_srp_client_host_addr_autoconfig() {
        log::error!("Error enabling srp client {e:?}");
    }

    openthread.set_srp_client_key_lease_interval(6800).ok();
    openthread.set_srp_client_lease_interval(720).ok();
    openthread.set_srp_client_ttl(30);


    loop {
        openthread.run_tasklets();
        openthread.process();
        
        critical_section::with(|cs| {
            let mut c = changed.borrow_ref_mut(cs);
            if c.0 {
                if c.1.contains(ChangedFlags::ThreadRlocAdded) {
                    println!("Attached to network, can now register SRP service");
                    register = true;
                }
                c.0 = false;
            }
        });

        if register {


            let state = openthread.get_srp_client_state();
            println!("SRP client state: {:?}", state);

            critical_section::with(|cs| {
                if let Err(e) = openthread.register_service_with_srp_client(
                    *SERVICENAME.borrow_ref(cs),
                    *INSTANCENAME.borrow_ref(cs),
                    *SUBTYPES.borrow_ref(cs),
                    *DNSTXT.borrow_ref(cs),
                    5683,
                    Some(1),
                    Some(1),
                    None,
                    None,
                ) {
                    log::error!("Error registering service {e:?}");
                } else {
                    println!(
                        "Registered SRP service; hostname {:?}, {:?}, {:?}",
                        *HOSTNAME.borrow_ref(cs),
                        *SERVICENAME.borrow_ref(cs),
                        *INSTANCENAME.borrow_ref(cs)
                    );
                }
            });

            break;
        } 
    }
        
    let state = openthread.get_srp_client_state();
    println!("SRP client state: {:?}", state);

    if let Err(e) = openthread.setup_srp_client_autostart(None) {
        log::error!("Error enabling srp client {e:?}");
    }

    let services = openthread.srp_get_services();

    for service in services {
        unsafe { println!("Service name: {:?}", core::ffi::CStr::from_ptr(service.name).to_str().unwrap()) };
        unsafe { println!("Instance name: {:?}", core::ffi::CStr::from_ptr(service.instance_name).to_str().unwrap()) };
        unsafe { println!("DNS key: {:?} value {:?}", core::ffi::CStr::from_ptr((*service.txt_entries).mKey).to_str().unwrap(), (*service.txt_entries).mValue) };

        println!("State: {:?}", service.state);
    }


    unsafe {asm!("fence");}


    // restrict scope of socket (so we can mutably borrow openthread after we break out of loop)
    {

        let state = openthread.get_srp_client_state();
        println!("SRP client state: {:?}", state);

        let mut socket = openthread.get_udp_socket::<512>().unwrap();
        let mut socket = pin!(socket);
        socket.bind(BOUND_PORT).unwrap();

        let mut buffer = [0u8; 512];
        println!("Dropping into UDP socket loop");
    
        loop {
            openthread.run_tasklets();
            openthread.process();
            

            let (len, from, port) = socket.receive(&mut buffer).unwrap();

            // When the program receives a UDP packet, it will unregister SRP services
            if len > 0 {
                println!(
                    "received {:02x?} from {:?} port {}",
                    &buffer[..len],
                    from,
                    port
                );
                socket.send(from, BOUND_PORT, b"Hello").unwrap();
                println!("Sent response message, now unregistering SRP service!");
                break;
            }

            critical_section::with(|cs| {
                let mut c = changed.borrow_ref_mut(cs);
                if c.0 {
                    let addrs: heapless::Vec<NetworkInterfaceUnicastAddress, 5> =
                        openthread.ipv6_get_unicast_addresses();

                    print_all_addresses(addrs);
                    c.0 = false;
                }
            });
        }
    }

    let state = openthread.get_srp_client_state();
    println!("SRP client state: {:?}", state);

    let services = openthread.srp_get_services();
    for service in services {
        unsafe { println!("Service name: {:?}", core::ffi::CStr::from_ptr(service.name).to_str().unwrap()) };
        unsafe { println!("Instance name: {:?}", core::ffi::CStr::from_ptr(service.instance_name).to_str().unwrap()) };
        println!("State: {:?}", service.state);
    }

    if let Err(e) = openthread.srp_unregister_all_services(true, true) {
        log::error!("Failure to unregister all services {e:?}");
    }

    println!("SRP services unregistered, no longer receiving UDP packets");

    let state = openthread.get_srp_client_state();
    println!("SRP client state: {:?}", state);

    let mut break_loop = false;
    loop {
        openthread.run_tasklets();
        openthread.process();
        

        critical_section::with(|cs| {
            let mut c = changed.borrow_ref_mut(cs);
            if c.0 {
                let addrs: heapless::Vec<NetworkInterfaceUnicastAddress, 5> =
                    openthread.ipv6_get_unicast_addresses();

                print_all_addresses(addrs);

                if c.1.contains(ChangedFlags::ThreadRlocRemoved) {
                    println!("Dropped from Thread network, resetting!");
                    break_loop = true;
                }
                c.0 = false;
            }
        });
        if break_loop {
            break;
        }
    }

    esp_hal::reset::software_reset_cpu();
    // This wont execute
    loop {}
}

fn print_all_addresses(addrs: heapless::Vec<NetworkInterfaceUnicastAddress, 5>) {
    println!("Currently assigned addresses");
    for addr in addrs {
        println!("{}", addr.address);
    }
    println!();
}

/*
fn print_services(services: heapless::Vec<NetworkInterfaceUnicastAddress, 5>) {
    println!("Currently assigned addresses");
    for addr in addrs {
        println!("{}", addr.address);
    }
    println!();
}*/ 