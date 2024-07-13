//! Full MTD impl example workspace

#![no_std]
#![no_main]

use core::cell::RefCell;
use core::pin::pin;
use core::ptr::addr_of_mut;

use critical_section::Mutex;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl, gpio::Io, peripherals::Peripherals, prelude::*, rng::Rng,
    system::SystemControl, timer::systimer,
};
use esp_ieee802154::{Config, Ieee802154};
use esp_openthread::ChangedFlags;
use esp_openthread::{NetworkInterfaceUnicastAddress, OperationalDataset, ThreadTimestamp};
use esp_println::println;

use esp_hal_smartled::{smartLedBuffer, SmartLedsAdapter};
use smart_leds::{brightness, colors, gamma, SmartLedsWrite};

pub const BOUND_PORT: u16 = 1212;

#[global_allocator]
static ALLOC: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

pub fn init_heap() {
    const SIZE: usize = 32768;
    static mut HEAP: [u8; SIZE] = [0; SIZE];
    unsafe { ALLOC.init(addr_of_mut!(HEAP) as *mut u8, SIZE) }
}

#[derive(Default)]
struct ChangeCallback {
    flags: ChangedFlags,
    changed: bool,
}

#[entry]
fn main() -> ! {
    esp_println::logger::init_logger(log::LevelFilter::Debug);

    init_heap();
    let mut peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    println!("Initializing");

    let systimer = systimer::SystemTimer::new(peripherals.SYSTIMER);
    let radio = peripherals.IEEE802154;
    let mut ieee802154 = Ieee802154::new(radio, &mut peripherals.RADIO_CLK);

    let mut openthread = esp_openthread::OpenThread::new(
        &mut ieee802154,
        systimer.alarm0,
        Rng::new(peripherals.RNG),
    );

    let changed = Mutex::new(RefCell::new(ChangeCallback::default()));
    let mut callback = |flags| {
        println!("{:?}", flags);
        critical_section::with(|cs| {
            let mut change = changed.borrow_ref_mut(cs);
            change.flags = flags;
            change.changed = true
        });
    };

    openthread
        .set_radio_config(Config {
            auto_ack_tx: true,
            auto_ack_rx: true,
            promiscuous: false,
            rx_when_idle: true,
            txpower: 18, // 18 txpower is legal for North America
            channel: 25, // match the dataset
            ..Config::default()
        })
        .unwrap();

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

    //openthread.set_child_timeout(60).unwrap();

    openthread.ipv6_set_enabled(true).unwrap();

    openthread.thread_set_enabled(true).unwrap();

    let addrs: heapless::Vec<NetworkInterfaceUnicastAddress, 6> =
        openthread.ipv6_get_unicast_addresses();

    print_all_addresses(addrs);

    let mut socket = openthread.get_udp_socket::<512>().unwrap();
    let mut socket = pin!(socket);
    socket.bind(BOUND_PORT).unwrap();

    let mut buffer = [0u8; 512];

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let led_pin = io.pins.gpio8;

    // Configure RMT peripheral globally
    #[cfg(not(feature = "esp32h2"))]
    let rmt = esp_hal::rmt::Rmt::new(peripherals.RMT, 80.MHz(), &clocks, None).unwrap();
    #[cfg(feature = "esp32h2")]
    let rmt = esp_hal::rmt::Rmt::new(peripherals.RMT, 32.MHz(), &clocks, None).unwrap();

    let rmt_buffer = smartLedBuffer!(1);
    let mut led = SmartLedsAdapter::new(rmt.channel0, led_pin, rmt_buffer, &clocks);
    let mut data;
    let mut eui: [u8; 6] = [0u8; 6];

    loop {
        openthread.process();
        openthread.run_tasklets();

        data = [colors::SEA_GREEN];
        led.write(brightness(gamma(data.iter().cloned()), 50))
            .unwrap();

        let (len, from, port) = socket.receive(&mut buffer).unwrap();
        if len > 0 {
            println!(
                "received {:02x?} from {:?} port {}",
                &buffer[..len],
                from,
                port
            );

            socket.send(from, BOUND_PORT, b"Hello").unwrap();
            println!("Sent message");
        }

        critical_section::with(|cs| {
            // let mut c = changed.borrow_ref_mut(cs);

            let mut c = changed.borrow_ref_mut(cs);
            //let c = change_callback.as_mut();

            if c.changed {
                let addrs: heapless::Vec<NetworkInterfaceUnicastAddress, 6> =
                    openthread.ipv6_get_unicast_addresses();

                print_all_addresses(addrs);
                let role = openthread.get_device_role();
                openthread.get_eui(&mut eui);
                println!("Role: {:?}, Eui {:#X?}", role, eui);
                c.changed = false;
            }
        });

        data = [colors::MEDIUM_ORCHID];
        led.write(brightness(gamma(data.iter().cloned()), 50))
            .unwrap();
    }
}

fn print_all_addresses(addrs: heapless::Vec<NetworkInterfaceUnicastAddress, 6>) {
    println!("Currently assigned addresses");
    for addr in addrs {
        println!("{}", addr.address);
    }
    println!();
}
