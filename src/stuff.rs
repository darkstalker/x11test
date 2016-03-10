use x11::{xlib, xinput2};
use std::{mem, slice};
use std::ffi::{CStr, CString};

pub fn desc_flags<'a>(flags: i32, desc: &'a [&str]) -> Vec<&'a str>
{
    (0 .. desc.len()).filter(|i| flags & (1 << i) != 0).map(|i| desc[i]).collect()
}

pub fn enumerate_devices(display: *mut xlib::Display)
{
    let mut ndevices = 0;
    let devices_ptr = unsafe { xinput2::XIQueryDevice(display, xinput2::XIAllDevices, &mut ndevices) };
    let xi_devices = unsafe { slice::from_raw_parts(devices_ptr, ndevices as usize) };

    let dev_use = ["unk", "XIMasterPointer", "XIMasterKeyboard", "XISlavePointer", "XISlaveKeyboard", "XIFloatingSlave"];
    let dev_class = ["XIKeyClass", "XIButtonClass", "XIValuatorClass", "XIScrollClass", "4", "5", "6", "7", "XITouchClass"];

    for dev in xi_devices
    {
        let name = unsafe{ CStr::from_ptr(dev.name) };
        println!("-- device {} {:?} is a {} (paired with {})", dev.deviceid, name, dev_use[dev._use as usize], dev.attachment);

        for i in 0 .. dev.num_classes as isize
        {
            let class = unsafe { *dev.classes.offset(i) };
            let _type = unsafe { (*class)._type };
            let ci_str = match _type {
                xinput2::XIKeyClass => {
                    let key_c: &xinput2::XIKeyClassInfo = unsafe { mem::transmute(class) };
                    format!("{} keycodes", key_c.num_keycodes)
                }
                xinput2::XIButtonClass => {
                    let but_c: &xinput2::XIButtonClassInfo = unsafe { mem::transmute(class) };
                    let atoms = unsafe { slice::from_raw_parts(but_c.labels, but_c.num_buttons as usize) };
                    let buttons: Vec<(_, _)> = atoms.iter().map(|&atom| {
                        if atom != 0
                        {
                            unsafe {
                                let ptr = xlib::XGetAtomName(display, atom);
                                let val = CStr::from_ptr(ptr).to_owned();
                                xlib::XFree(ptr as *mut _);
                                val
                            }
                        }
                        else { CString::new("(null)").unwrap() }
                    }).enumerate().map(|(i, s)| (i+1, s)).collect();
                    format!("{} buttons {:?}", but_c.num_buttons, buttons)
                },
                xinput2::XIValuatorClass => {
                    let val_c: &xinput2::XIValuatorClassInfo = unsafe { mem::transmute(class) };
                    let name = if val_c.label != 0 {
                        unsafe {
                            let ptr = xlib::XGetAtomName(display, val_c.label);
                            let val = CStr::from_ptr(ptr).to_owned();
                            xlib::XFree(ptr as *mut _);
                            val
                        }
                    }
                    else { CString::new("(null)").unwrap() };
                    format!("number {}, name {:?}, min {}, max {}, res {}, mode {}",
                        val_c.number, name, val_c.min, val_c.max, val_c.resolution, val_c.mode)
                },
                xinput2::XIScrollClass => {
                    let scr_c: &xinput2::XIScrollClassInfo = unsafe { mem::transmute(class) };
                    format!("number {}, stype {}, incr {}, flags={}", scr_c.number, scr_c.scroll_type, scr_c.increment, scr_c.flags)
                },
                xinput2::XITouchClass => {
                    let tou_c: &xinput2::XITouchClassInfo = unsafe { mem::transmute(class) };
                    format!("mode {}, num_touches {}", tou_c.mode, tou_c.num_touches)
                },
                _ => unreachable!()
            };
            println!("   class: {} ({})", dev_class[_type as usize], ci_str);
        }
    }

    unsafe{ xinput2::XIFreeDeviceInfo(devices_ptr); }
    println!("---");
}
