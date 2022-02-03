use std::{thread::sleep, time::Duration};

use rusb::{request_type, Context, Device, DeviceHandle, Interface, Result, UsbContext};

mod g35;

fn main() -> Result<()> {
    let mut context = Context::new()?;
    let (mut device, mut handle) = open_device(&mut context, g35::VENDOR_ID, g35::PRODUCT_ID)
        .expect("Failed to open USB device");

    print_device_info(&mut handle)?;
    control_transfer(&mut handle)?;
    Ok(())
}

fn open_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Option<(Device<T>, DeviceHandle<T>)> {
    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return None,
    };

    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };

        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            match device.open() {
                Ok(handle) => return Some((device, handle)),
                Err(_) => continue,
            }
        }
    }

    None
}

fn print_device_info<T: UsbContext>(handle: &mut DeviceHandle<T>) -> Result<()> {
    let device_desc = handle.device().device_descriptor()?;
    let timeout = std::time::Duration::from_secs(1);
    let languages = handle.read_languages(timeout)?;

    let active_config = handle.device().active_config_descriptor()?;

    println!(
        "Device address: {}.{}",
        handle.device().bus_number(),
        handle.device().address()
    );

    println!(
        "Possible configurations: {}",
        device_desc.num_configurations()
    );

    println!("Active configuration: {}", active_config.number());

    let interface_indexes: Vec<u8> = active_config
        .interfaces()
        .into_iter()
        .map(|x: Interface| x.number())
        .collect();
    println!("Configuration interface indexes: {:?}", interface_indexes);

    // for interface in active_config.interfaces().into_iter() {
    //     println!("Interface {}:", interface.number());

    //     for descriptor in interface.descriptors().into_iter() {
    //         println!("\tDescriptor {}", descriptor.class_code());
    //     }
    // }

    if !languages.is_empty() {
        let language = languages[0];
        println!("Language: {:?}", language);

        println!(
            "Manufacturer: {}",
            handle
                .read_manufacturer_string(language, &device_desc, timeout)
                .unwrap_or("Not Found".to_string())
        );
        println!(
            "Product: {}",
            handle
                .read_product_string(language, &device_desc, timeout)
                .unwrap_or("Not Found".to_string())
        );
        println!(
            "Serial Number: {}",
            handle
                .read_serial_number_string(language, &device_desc, timeout)
                .unwrap_or("Not Found".to_string())
        );
    }

    Ok(())
}

fn control_transfer<T: UsbContext>(device_handle: &mut DeviceHandle<T>) -> Result<()> {
    device_handle.set_auto_detach_kernel_driver(false)?;

    let device_descriptor = device_handle.device().device_descriptor()?;
    let active_config = device_handle.device().active_config_descriptor()?;

    let active_interfaces: Vec<Interface> = active_config.interfaces().collect::<Vec<_>>();

    let interface_to_claim = &active_interfaces[0];
    let interface_number_to_claim = interface_to_claim.number();
    //let interface_number_to_claim = ;

    let mut kernel_was_attached = false;
    if device_handle.kernel_driver_active(interface_number_to_claim)? {
        println!(
            "Kernel driver is active on interface {}, will try to detach.",
            interface_number_to_claim
        );
        device_handle.detach_kernel_driver(interface_number_to_claim)?;
        kernel_was_attached = true;
        println!("Kernel driver was detached!");
    }

    if device_handle
        .kernel_driver_active(interface_number_to_claim)
        .unwrap_or(false)
    {
        println!(
            "Kernel has driver active on interface {}",
            interface_number_to_claim
        );
    }

    println!("Attaching to interface {}...", interface_number_to_claim);

    device_handle.claim_interface(interface_number_to_claim)?;

    println!("Claimed interface!");

    sleep(Duration::from_millis(1));

    let sidetone_half = [0x00, 0xf4];

    println!("Writing control data...");

    device_handle.write_control(
        request_type(
            rusb::Direction::Out,
            rusb::RequestType::Class,
            rusb::Recipient::Interface,
        ),
        1,
        0x0200,
        1536,
        &sidetone_half,
        Duration::from_millis(1000),
    )?;

    println!("Wrote control data!");

    // let sleep_len = 3;
    // println!("Sleeping for {} seconds...", sleep_len);
    // sleep(Duration::from_secs(sleep_len));

    // if kernel_was_attached {
    //     println!("Restoring kernel driver...");
    //     device_handle.attach_kernel_driver(interface_number_to_claim)?;
    //     println!("Kernel driver was restored!");
    // }

    Ok(())
}
