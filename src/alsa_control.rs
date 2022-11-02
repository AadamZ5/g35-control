use alsa::{device_name::Hint, Direction};
use colored::{ColoredString, Colorize};
use std::{
    error,
    ffi::{CStr, CString},
    io::Write,
    process::exit,
};

type Res<T> = Result<T, Box<dyn error::Error>>;

#[derive(Clone)]
struct DeviceOption {
    primary_name: ColoredString,
    secondary_name: Option<String>,
    tech_info: String,
    enumeration_option: u16,
    direction: Option<Direction>,
    hint: Hint,
}

impl DeviceOption {
    fn new(
        primary_name: ColoredString,
        secondary_name: Option<String>,
        tech_info: String,
        enumeration_option: u16,
        direction: Option<Direction>,
        from_hint: Hint,
    ) -> Self {
        Self {
            primary_name,
            secondary_name,
            tech_info,
            enumeration_option,
            direction,
            hint: from_hint,
        }
    }
}

pub fn do_the_thing() -> Res<()> {
    // Get options
    let hint_list = get_initial_device_options()?;
    let device = get_device_selection(hint_list)?;

    if device.is_none() {
        println!("{}", "Device not found!".bright_red());
        exit(1);
    }

    let device = device.unwrap();

    println!(
        "Device found: {}{} - {}",
        device.primary_name,
        device.secondary_name.unwrap_or("".to_string()),
        device.tech_info
    );

    let dev_name = CString::new(device.tech_info)?;
    let ctl_iface = alsa::ctl::Ctl::open(dev_name.as_c_str(), false)?;

    println!(
        "Opened control interface {} {}",
        ctl_iface.card_info()?.get_name()?.white().bold(),
        "✓".green()
    );

    let ctl_info_binding = ctl_iface.card_info().ok();
    let ctl_info = ctl_info_binding.as_ref();
    let ctl_parts = match ctl_info {
        Some(info) => {
            let name = info.get_name().unwrap_or("Unknown");
            let id = info.get_id().unwrap_or("Unknown");
            let driver = info.get_driver().unwrap_or("Unknown");
            let mixername = info.get_mixername().unwrap_or("Unknown");
            let components = info.get_components().unwrap_or("Unknown");
            Some((name, id, driver, mixername, components))
        }
        None => None,
    };

    print!("\n");

    if let Some((name, id, driver, mixername, components)) = ctl_parts {
        println!("Name: {}", name.white());
        println!("ID: {}", id.white());
        println!("Driver: {}", driver.white());
        println!("Mixername: {}", mixername.white());
        println!("Components: {}", components.white());
    } else {
        println!("Unable to print control interface information.");
    }

    Ok(())
}

fn get_initial_device_options() -> Res<Vec<DeviceOption>> {
    /*The context to get a list from */
    let context = CStr::from_bytes_with_nul(b"ctl\0")?;

    let mut hint_list: Vec<DeviceOption> = alsa::device_name::HintIter::new(None, &context)?
        .map(|h| {
            let formatted_name_parts = match h.clone().desc {
                Some(name) => {
                    let name_parts: Vec<String> = name.split('\n').map(|s| s.to_string()).collect();
                    let primary_name = name_parts[0].white().bold();

                    let remaining = name_parts[1..]
                        .into_iter()
                        .map(|s| s.white().to_string())
                        .collect::<Vec<String>>()
                        .join(" | ".white().to_string().as_str());
                    (
                        primary_name,
                        Some(" | ".white().to_string() + remaining.as_str()),
                    )
                }
                None => ("Unknown".bright_yellow().italic(), None),
            };

            let tech_name = match h.clone().name {
                Some(name) => name,
                None => "Unknown".italic().to_string(),
            };

            DeviceOption::new(
                formatted_name_parts.0,
                formatted_name_parts.1,
                tech_name,
                0,
                h.direction,
                h.clone(),
            )
        })
        .collect::<Vec<DeviceOption>>();

    for i in 0..hint_list.len() {
        hint_list[i].enumeration_option = (i + 1) as u16;
    }

    return Ok(hint_list);
}

fn get_device_selection(device_list: Vec<DeviceOption>) -> Res<Option<DeviceOption>> {
    let stdin = std::io::stdin();
    let mut stdio = std::io::stdout();

    // Show options nice and pretty
    for dev in device_list.clone() {
        println!(
            "{}. {}{} - {}",
            dev.enumeration_option.to_string().cyan().bold(),
            dev.primary_name,
            dev.secondary_name.unwrap_or("".to_string()),
            dev.tech_info
        );
    }
    println!(
        "Listed {} available devices",
        device_list.len().to_string().white()
    );

    // Get user input for which device they wanna target
    print!(
        "Enter device {} or {} to control: ",
        "name".white(),
        "number".cyan()
    );
    stdio.flush()?;
    let mut input = String::new();
    stdin.read_line(&mut input)?;
    let device_name_input = input.trim();
    let device_num_input = str::parse::<u16>(device_name_input).ok();

    // Find the device they want by the name they supplied
    let device = device_list
        .iter()
        .find(|dev| {
            let compare_name = dev.primary_name.clone().clear().to_string();
            let compare_name = compare_name.trim();
            return match device_num_input {
                Some(num) => num == dev.enumeration_option || compare_name == device_name_input,
                None => compare_name == device_name_input,
            };
        })
        .map(|dev| dev.to_owned());

    return Ok(device);
}
