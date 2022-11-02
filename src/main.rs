mod alsa_control;
mod g35;
mod usb_control;

fn main() -> Result<(), String> {
    alsa_control::do_the_thing().map_err(|e| e.to_string())?;
    //usb_control::do_the_thing().map_err(|e| e.to_string())?;
    Ok(())
}
