extern crate x11;
extern crate glengine;

pub mod event;

use x11::{xlib, xinput2, keysym};
use std::{mem, slice};
use std::ptr;
use std::ffi::CString;
use std::collections::{hash_map, HashMap, VecDeque};
use std::rc::{Rc, Weak};
use std::cell::{Cell, RefCell};

pub use event::*;

fn as_button(button: i32) -> Button
{
    match button {
        1 => Button::Left,
        2 => Button::Middle,
        3 => Button::Right,
        other => Button::Other(other as u8)
    }
}

fn as_axis_scroll(button: i32) -> AxisState
{
    match button {
        4 => AxisState::Scroll(0.0, 1.0),
        5 => AxisState::Scroll(0.0, -1.0),
        6 => AxisState::Scroll(-1.0, 0.0),
        7 => AxisState::Scroll(1.0, 0.0),
        _ => unreachable!()
    }
}

fn convert_keysym(keysym: u32) -> Key
{
    match keysym {
        keysym::XK_0 => Key::Key0,
        keysym::XK_1 => Key::Key1,
        keysym::XK_2 => Key::Key2,
        keysym::XK_3 => Key::Key3,
        keysym::XK_4 => Key::Key4,
        keysym::XK_5 => Key::Key5,
        keysym::XK_6 => Key::Key6,
        keysym::XK_7 => Key::Key7,
        keysym::XK_8 => Key::Key8,
        keysym::XK_9 => Key::Key9,

        keysym::XK_a => Key::A,
        keysym::XK_b => Key::B,
        keysym::XK_c => Key::C,
        keysym::XK_d => Key::D,
        keysym::XK_e => Key::E,
        keysym::XK_f => Key::F,
        keysym::XK_g => Key::G,
        keysym::XK_h => Key::H,
        keysym::XK_i => Key::I,
        keysym::XK_j => Key::J,
        keysym::XK_k => Key::K,
        keysym::XK_l => Key::L,
        keysym::XK_m => Key::M,
        keysym::XK_n => Key::N,
        keysym::XK_o => Key::O,
        keysym::XK_p => Key::P,
        keysym::XK_q => Key::Q,
        keysym::XK_r => Key::R,
        keysym::XK_s => Key::S,
        keysym::XK_t => Key::T,
        keysym::XK_u => Key::U,
        keysym::XK_v => Key::V,
        keysym::XK_w => Key::W,
        keysym::XK_x => Key::X,
        keysym::XK_y => Key::Y,
        keysym::XK_z => Key::Z,

        keysym::XK_F1 => Key::F1,
        keysym::XK_F2 => Key::F2,
        keysym::XK_F3 => Key::F3,
        keysym::XK_F4 => Key::F4,
        keysym::XK_F5 => Key::F5,
        keysym::XK_F6 => Key::F6,
        keysym::XK_F7 => Key::F7,
        keysym::XK_F8 => Key::F8,
        keysym::XK_F9 => Key::F9,
        keysym::XK_F10 => Key::F10,
        keysym::XK_F11 => Key::F11,
        keysym::XK_F12 => Key::F12,

        keysym::XK_Escape => Key::Escape,
        keysym::XK_BackSpace => Key::BackSpace,
        keysym::XK_Tab => Key::Tab,
        keysym::XK_Return => Key::Return,
        keysym::XK_Caps_Lock => Key::CapsLock,
        keysym::XK_Shift_L => Key::ShiftLeft,
        keysym::XK_Shift_R => Key::ShiftRight,
        keysym::XK_Control_L => Key::ControlLeft,
        keysym::XK_Control_R => Key::ControlRight,
        keysym::XK_Alt_L => Key::AltLeft,
        keysym::XK_Alt_R => Key::AltRight,  // US only
        keysym::XK_Super_L => Key::SuperLeft,
        keysym::XK_Super_R => Key::SuperRight,
        keysym::XK_Meta_L => Key::MetaLeft, // Mac's ⌘ key
        keysym::XK_Meta_R => Key::MetaRight,
        keysym::XK_Mode_switch => Key::ModeSwitch,  // Mac's ⌥ key
        keysym::XK_space => Key::Space,

        keysym::XK_Print => Key::Print,
        keysym::XK_Scroll_Lock => Key::ScrollLock,
        keysym::XK_Pause => Key::Pause,
        keysym::XK_Insert => Key::Insert,
        keysym::XK_Delete => Key::Delete,
        keysym::XK_Home => Key::Home,
        keysym::XK_End => Key::End,
        keysym::XK_Page_Up => Key::PageUp,
        keysym::XK_Page_Down => Key::PageDown,
        keysym::XK_Up => Key::Up,
        keysym::XK_Down => Key::Down,
        keysym::XK_Right => Key::Right,
        keysym::XK_Left => Key::Left,
        // on my system it doesn't send numpad numbers here..
        keysym::XK_KP_0 => Key::Numpad0,
        keysym::XK_KP_1 => Key::Numpad1,
        keysym::XK_KP_2 => Key::Numpad2,
        keysym::XK_KP_3 => Key::Numpad3,
        keysym::XK_KP_4 => Key::Numpad4,
        keysym::XK_KP_5 => Key::Numpad5,
        keysym::XK_KP_6 => Key::Numpad6,
        keysym::XK_KP_7 => Key::Numpad7,
        keysym::XK_KP_8 => Key::Numpad8,
        keysym::XK_KP_9 => Key::Numpad9,
        // .. but it does here
        keysym::XK_KP_Insert => Key::Numpad0,
        keysym::XK_KP_End => Key::Numpad1,
        keysym::XK_KP_Down => Key::Numpad2,
        keysym::XK_KP_Page_Down => Key::Numpad3,
        keysym::XK_KP_Left => Key::Numpad4,
        keysym::XK_KP_Begin => Key::Numpad5,
        keysym::XK_KP_Right => Key::Numpad6,
        keysym::XK_KP_Home => Key::Numpad7,
        keysym::XK_KP_Up => Key::Numpad8,
        keysym::XK_KP_Page_Up => Key::Numpad9,

        keysym::XK_Num_Lock => Key::NumLock,
        keysym::XK_KP_Delete => Key::NumpadDelete,
        keysym::XK_KP_Add => Key::NumpadAdd,
        keysym::XK_KP_Subtract => Key::NumpadSubtract,
        keysym::XK_KP_Multiply => Key::NumpadMultiply,
        keysym::XK_KP_Divide => Key::NumpadDivide,
        keysym::XK_KP_Enter => Key::NumpadEnter,

        keysym::XK_numbersign => Key::Numbersign,
        keysym::XK_apostrophe => Key::Apostrophe,
        keysym::XK_plus => Key::Plus,
        keysym::XK_comma => Key::Comma,
        keysym::XK_minus => Key::Minus,
        keysym::XK_period => Key::Period,
        keysym::XK_slash => Key::Slash,
        keysym::XK_semicolon => Key::Semicolon,
        keysym::XK_less => Key::Less,
        keysym::XK_equal => Key::Equal,
        keysym::XK_bracketleft => Key::BracketLeft,
        keysym::XK_backslash => Key::BackSlash,
        keysym::XK_bracketright => Key::BracketRight,
        keysym::XK_grave => Key::Grave,
        keysym::XK_braceleft => Key::BraceLeft,
        keysym::XK_bar => Key::Bar,
        keysym::XK_braceright => Key::BraceRight,
        keysym::XK_exclamdown => Key::ExclamDown,
        keysym::XK_guillemotleft => Key::GuillemotLeft,
        keysym::XK_masculine => Key::Masculine,
        keysym::XK_questiondown => Key::QuestionDown,
        keysym::XK_agrave => Key::AGrave,
        keysym::XK_ccedilla => Key::CCedilla,
        keysym::XK_egrave => Key::EGrave,
        keysym::XK_eacute => Key::EAcute,
        keysym::XK_igrave => Key::IGrave,
        keysym::XK_ntilde => Key::NTilde,
        keysym::XK_ograve => Key::OGrave,
        keysym::XK_ugrave => Key::UGrave,
        // things not in x11::keysym
        0xfe03 /* XK_ISO_Level3_Shift */ => Key::AltRight,  // AltGr -- Non-US sends this instead of Alt_R
        0xfe50 /* XK_dead_grave */ => Key::DeadGrave,
        0xfe51 /* XK_dead_acute */ => Key::DeadAcute,
        0xfe52 /* XK_dead_circumflex */ => Key::DeadCircumflex,
        0xfe53 /* XK_dead_tilde */ => Key::DeadTilde,
        0xfe5b /* XK_dead_cedilla */ => Key::DeadCedilla,
        other => Key::Unk(other)
    }
}

enum ParsedEvent
{
    None,
    One(Event),
    Many(Vec<Event>),
}

impl Default for ParsedEvent
{
    fn default() -> Self
    {
        ParsedEvent::None
    }
}

#[derive(Debug)]
enum AxisType
{
    ScrollVertical(f64 /* increment */),
    ScrollHorizontal(f64 /* increment */),
    Pressure(f64 /* max */),
    TiltX(f64 /* max */),
    TiltY(f64 /* max */),
}

#[derive(Debug)]
struct AxisData
{
    axis_type: AxisType,
    value: f64,
}

#[derive(Debug, Default)]
struct DeviceInfo
{
    axis_info: HashMap<i32 /* axis_num */, AxisData>,
    num_axis: i32,
    has_scroll: bool,
}

#[repr(C)]
struct AtomCache
{
    wm_delete_window: xlib::Atom,
    wm_protocols: xlib::Atom,
    abs_pressure: xlib::Atom,
    abs_tilt_x: xlib::Atom,
    abs_tilt_y: xlib::Atom,
}

pub struct XDisplay
{
    handle: *mut xlib::Display,
    win_data: RefCell<HashMap<xlib::Window, Weak<WindowData>>>,
    devices: RefCell<HashMap<i32 /* device_id */, DeviceInfo>>,
    pointer_pos: Cell<(f64, f64)>,
    atoms: AtomCache,
    engine: glengine::DrawEngine,
}

impl XDisplay
{
    pub fn new() -> Result<Self, &'static str>  //TODO: Err type
    {
        // open display
        let display = unsafe{ xlib::XOpenDisplay(ptr::null()) };
        if display == ptr::null_mut()
        {
            return Err("Can't open display")
        }

        // this initializes EGL and the GL context
        let engine = try!(glengine::DrawEngine::new(display as _));

        let mut xdis = XDisplay{
            handle: display,
            win_data: Default::default(),
            devices: Default::default(),
            pointer_pos: Cell::new((-1.0, -1.0)),
            atoms: unsafe { mem::zeroed() },
            engine: engine,
        };

        // get atoms
        let mut atom_names = [b"WM_DELETE_WINDOW\0".as_ptr() as *mut _,
                              b"WM_PROTOCOLS\0".as_ptr() as *mut _,
                              b"Abs Pressure\0".as_ptr() as *mut _,
                              b"Abs Tilt X\0".as_ptr() as *mut _,
                              b"Abs Tilt Y\0".as_ptr() as *mut _];
        if unsafe { xlib::XInternAtoms(display, &mut atom_names[0], atom_names.len() as i32, xlib::False, mem::transmute(&mut xdis.atoms)) } == 0
        {
            return Err("Can't intern atoms")
        }

        // query for XInput support
        let mut ex_opcode = 0;
        let mut ex_event = 0;
        let mut ex_error = 0;
        if unsafe { xlib::XQueryExtension(display, b"XInputExtension\0".as_ptr() as *const _,
            &mut ex_opcode, &mut ex_event, &mut ex_error) } == xlib::False
        {
            return Err("XInput extension unavailable")
        }

        // check XInput version
        let mut xi_major = xinput2::XI_2_Major;
        let mut xi_minor = xinput2::XI_2_Minor;
        if unsafe { xinput2::XIQueryVersion(display, &mut xi_major, &mut xi_minor) } != xlib::Success as i32
        {
            return Err("XInput2 not available")
        }

        // enable XInput hierarchy events
        let mut mask = [0; 2];
        xinput2::XISetMask(&mut mask, xinput2::XI_HierarchyChanged);

        let mut event_mask = xinput2::XIEventMask{
            deviceid: xinput2::XIAllDevices,
            mask_len: mask.len() as i32,
            mask: mask.as_mut_ptr(),
        };

        let root_win = unsafe { xlib::XDefaultRootWindow(display) };
        if unsafe { xinput2::XISelectEvents(display, root_win, &mut event_mask, 1) } != xlib::Success as i32
        {
            return Err("Failed to select HierarchyChanged event")
        }

        // disable fake KeyRelease events on auto repeat
        unsafe { xlib::XkbSetDetectableAutoRepeat(display, xlib::True, ptr::null_mut()); }

        // load device info
        xdis.load_axis_info(xinput2::XIAllDevices);

        Ok(xdis)
    }

    pub fn create_window(&self, width: u32, height: u32) -> Result<XWindow, &'static str>
    {
        XWindow::new(self, width, height)
    }

    pub fn wait_event(&self)
    {
        let mut xevent = unsafe { mem::zeroed() };
        loop
        {
            unsafe{ xlib::XNextEvent(self.handle, &mut xevent); }

            let (win, parse_res) = self.parse_event(xevent);

            let got_event = self.with_windata(win, move |wd| {
                match parse_res {
                    ParsedEvent::One(event) => {
                        wd.ev_queue.borrow_mut().push_back(event);
                        true
                    },
                    ParsedEvent::Many(events) => {
                        wd.ev_queue.borrow_mut().extend(events);
                        true
                    },
                    ParsedEvent::None => false,
                }
            });

            if got_event { break }
        }
    }

    fn parse_event(&self, mut xevent: xlib::XEvent) -> (xlib::Window, ParsedEvent)
    {
        match xevent.get_type() {
            xlib::KeyPress => {
                let ev: &xlib::XKeyPressedEvent = xevent.as_ref();
                let key = self.scancode_to_key(ev.keycode as u8);
                (ev.window, ParsedEvent::One(Event::Keyboard(EvState::Pressed, key)))
            },
            xlib::KeyRelease => {
                let ev: &xlib::XKeyReleasedEvent = xevent.as_ref();
                let key = self.scancode_to_key(ev.keycode as u8);
                (ev.window, ParsedEvent::One(Event::Keyboard(EvState::Released, key)))
            },
            xlib::EnterNotify => {
                let ev: &xlib::XEnterWindowEvent = xevent.as_ref();
                if ev.mode == xlib::NotifyNormal ||
                  (ev.mode == xlib::NotifyUngrab && ev.detail == xlib::NotifyNonlinear)
                {
                    // if the mouse has been outside, we need to reload the absolute value of the scroll axis
                    self.reload_scroll_values();
                    (ev.window, ParsedEvent::One(Event::PointerInside(true)))
                }
                else { (ev.window, ParsedEvent::None) }
            },
            xlib::LeaveNotify => {
                let ev: &xlib::XLeaveWindowEvent = xevent.as_ref();
                match ev.mode {
                    xlib::NotifyNormal => (ev.window, ParsedEvent::One(Event::PointerInside(false))),
                    _ => (ev.window, ParsedEvent::None)
                }
            },
            xlib::FocusIn => {
                let ev: &xlib::XFocusInEvent = xevent.as_ref();
                match ev.mode {
                    xlib::NotifyNormal | xlib::NotifyWhileGrabbed => (ev.window, ParsedEvent::One(Event::Focused(true))),
                    _ => (ev.window, ParsedEvent::None)
                }
            },
            xlib::FocusOut => {
                let ev: &xlib::XFocusOutEvent = xevent.as_ref();
                match ev.mode {
                    xlib::NotifyNormal | xlib::NotifyWhileGrabbed => (ev.window, ParsedEvent::One(Event::Focused(false))),
                    _ => (ev.window, ParsedEvent::None)
                }
            },
            xlib::Expose => {
                let ev: &xlib::XExposeEvent = xevent.as_ref();
                (ev.window, ParsedEvent::One(Event::Redraw))
            },
            xlib::ConfigureNotify => {
                let ev: &xlib::XConfigureEvent = xevent.as_ref();
                (ev.window, self.with_windata(ev.window, |wd| {
                    let mut events = Vec::with_capacity(2);
                    let size = (ev.width as u32, ev.height as u32);
                    if wd.size.get() != size
                    {
                        wd.size.set(size);
                        events.push(Event::Resized(size.0, size.1));
                    }

                    if ev.above == 0  // when .above is set, event contains a bogus position value
                    {
                        let pos = (ev.x, ev.y);
                        if wd.pos.get() != pos
                        {
                            wd.pos.set(pos);
                            events.push(Event::Moved(pos.0, pos.1));
                        }
                    }

                    if !events.is_empty()
                    {
                        ParsedEvent::Many(events)
                    }
                    else { ParsedEvent::None }
                }))
            },
            xlib::ClientMessage => {
                let ev: &xlib::XClientMessageEvent = xevent.as_ref();
                if ev.message_type == self.atoms.wm_protocols && ev.format == 32 &&
                    (ev.data.get_long(0) as xlib::Atom) == self.atoms.wm_delete_window
                {
                    (ev.window, ParsedEvent::One(Event::CloseButton))
                }
                else { (ev.window, ParsedEvent::None) }
            },
            xlib::GenericEvent => {
                let ev: &mut xlib::XGenericEventCookie = xevent.as_mut();
                if unsafe { xlib::XGetEventData(self.handle, ev) } == xlib::False
                {
                    println!("failed to get event data");
                    return (0, ParsedEvent::None);
                }

                let event = self.parse_xinput_event(ev);

                unsafe { xlib::XFreeEventData(self.handle, ev); }
                event
            },
            _ => {
                let ev: &xlib::XAnyEvent = xevent.as_ref();
                (ev.window, ParsedEvent::None)
            }
        }
    }

    fn parse_xinput_event(&self, ev: &xlib::XGenericEventCookie) -> (xlib::Window, ParsedEvent)
    {
        let ev_data: &xinput2::XIDeviceEvent = unsafe { mem::transmute(ev.data) };

        (ev_data.event, match ev.evtype {
            /*xinput2::XI_DeviceChanged => {
                //let ev_data: &xinput2::XIDeviceChangedEvent = unsafe { mem::transmute(ev.data) };
                println!("-- device changed!");
                ParsedEvent::None
            },*/
            xinput2::XI_ButtonPress | xinput2::XI_ButtonRelease => {
                let button_id = ev_data.detail;
                let state = if ev.evtype == xinput2::XI_ButtonPress { EvState::Pressed } else { EvState::Released };
                if button_id >= 4 && button_id <= 7  // is wheel
                {
                    if ev_data.flags & xinput2::XIPointerEmulated != 0  // emulated event, real data is in XI_Motion
                    {
                        ParsedEvent::None
                    }
                    else
                    {
                        ParsedEvent::One(Event::AxisMoved(as_axis_scroll(button_id)))
                    }
                }
                else
                {
                    ParsedEvent::One(Event::MouseButton(state, as_button(button_id), (ev_data.event_x, ev_data.event_y)))
                }
            },
            xinput2::XI_Motion => {
                let axis_state = ev_data.valuators;
                let axis_mask = unsafe{ slice::from_raw_parts(axis_state.mask, axis_state.mask_len as usize) };

                let (mut scroll_x, mut scroll_y) = (0.0, 0.0);
                let mut pressure = None;
                let (mut tilt_x_changed, mut tilt_y_changed) = (false, false);
                let (mut tilt_x, mut tilt_y) = (0.0, 0.0);

                let mut devices = self.devices.borrow_mut();
                let dev_info = devices.get_mut(&ev_data.sourceid).unwrap();

                let mut cur_offset = 0;
                for axis_id in 0 .. dev_info.num_axis
                {
                    if xinput2::XIMaskIsSet(&axis_mask, axis_id)
                    {
                        let axis_value = unsafe { *axis_state.values.offset(cur_offset) };
                        if let Some(axis_info) = dev_info.axis_info.get_mut(&axis_id)
                        {
                            match axis_info.axis_type {
                                AxisType::ScrollVertical(incr) => if axis_info.value != axis_value
                                {
                                    scroll_y = (axis_info.value - axis_value) / incr;
                                },
                                AxisType::ScrollHorizontal(incr) => if axis_info.value != axis_value
                                {
                                    scroll_x = (axis_info.value - axis_value) / incr;
                                },
                                AxisType::Pressure(max) => if axis_info.value != axis_value
                                {
                                    pressure = Some(axis_value / max);
                                },
                                // assuming those two are always present in pairs
                                AxisType::TiltX(max) => {
                                    let val = if axis_info.value != axis_value
                                    {
                                        tilt_x_changed = true;
                                        axis_value
                                    }
                                    else
                                    {
                                        axis_info.value
                                    };
                                    tilt_x = (val / max).min(1.0).max(-1.0);
                                },
                                AxisType::TiltY(max) => {
                                    let val = if axis_info.value != axis_value
                                    {
                                        tilt_y_changed = true;
                                        axis_value
                                    }
                                    else
                                    {
                                        axis_info.value
                                    };
                                    tilt_y = (val / max).min(1.0).max(-1.0);
                                },
                            }

                            axis_info.value = axis_value;
                        }

                        cur_offset += 1;
                    }
                }

                let mut events = Vec::with_capacity(4);

                let pointer_pos = (ev_data.root_x, ev_data.root_y);
                if self.pointer_pos.get() != pointer_pos
                {
                    self.pointer_pos.set(pointer_pos);
                    events.push(Event::MouseMoved(ev_data.event_x, ev_data.event_y));
                }
                if scroll_x != 0.0 || scroll_y != 0.0
                {
                    events.push(Event::AxisMoved(AxisState::Scroll(scroll_x, scroll_y)))
                }
                if let Some(val) = pressure
                {
                    events.push(Event::AxisMoved(AxisState::Pressure(val)))
                }
                if tilt_x_changed || tilt_y_changed
                {
                    events.push(Event::AxisMoved(AxisState::Tilt(tilt_x, tilt_y)))
                }

                ParsedEvent::Many(events)
            },
            xinput2::XI_HierarchyChanged => {
                let ev_data: &xinput2::XIHierarchyEvent = unsafe { mem::transmute(ev.data) };
                let ev_info = unsafe { slice::from_raw_parts(ev_data.info, ev_data.num_info as usize) };

                if ev_data.flags & (xinput2::XIDeviceEnabled | xinput2::XIDeviceDisabled) != 0
                {
                    for info in ev_info
                    {
                        if info._use == xinput2::XISlavePointer
                        {
                            if info.flags & xinput2::XIDeviceEnabled != 0
                            {
                                println!("** adding device: {}", info.deviceid);
                                self.load_axis_info(info.deviceid);
                            }
                            else if info.flags & xinput2::XIDeviceDisabled != 0
                            {
                                self.devices.borrow_mut().remove(&info.deviceid);
                                println!("** removed device: {}", info.deviceid);
                            }
                        }
                    }
                }

                return (0, ParsedEvent::None)
            },
            _ => return (0, ParsedEvent::None)
        })
    }

    fn load_axis_info(&self, device_id: i32)
    {
        let mut ndevices = 0;
        let devices_ptr = unsafe { xinput2::XIQueryDevice(self.handle, device_id, &mut ndevices) };
        let xi_devices = unsafe { slice::from_raw_parts(devices_ptr, ndevices as usize) };

        for dev in xi_devices
        {
            // slave pointers are the physical devices that can have multiple axis
            if dev._use == xinput2::XISlavePointer
            {
                println!("-- device {}", dev.deviceid);

                let mut values = vec![0.0; dev.num_classes as usize];
                let mut has_scroll = false;

                for i in 0 .. dev.num_classes as isize
                {
                    let class = unsafe { *dev.classes.offset(i) };
                    let ctype = unsafe { (*class)._type };

                    let (ax_num, ax_data) = match ctype {
                        xinput2::XIValuatorClass => {
                            let ci: &xinput2::XIValuatorClassInfo = unsafe { mem::transmute(class) };
                            println!("valuator {}: mode={} min={} max={} val={}", ci.number, ci.mode, ci.min, ci.max, ci.value);
                            // we're gonna assume valuators appear before scroll classes, so we can store them here ...
                            values[ci.number as usize] = ci.value;

                            let ax_type = match ci.label {
                                a if a == self.atoms.abs_pressure => AxisType::Pressure(ci.max),
                                // the tilt value should be "almost" symmetric (like -64 to 63)
                                a if a == self.atoms.abs_tilt_x => AxisType::TiltX(ci.max),
                                a if a == self.atoms.abs_tilt_y => AxisType::TiltY(ci.max),
                                _ => continue
                            };
                            (ci.number, AxisData{ axis_type: ax_type, value: ci.value })
                        },
                        xinput2::XIScrollClass => {
                            let ci: &xinput2::XIScrollClassInfo = unsafe { mem::transmute(class) };
                            println!("scroll {}: type={} incr={} flags={}", ci.number, ci.scroll_type, ci.increment, ci.flags);
                            has_scroll = true;
                            // ... and use them here to fill in the scroll classes
                            match ci.scroll_type {
                                xinput2::XIScrollTypeVertical => {
                                    (ci.number, AxisData{ axis_type: AxisType::ScrollVertical(ci.increment), value: values[ci.number as usize] })
                                },
                                xinput2::XIScrollTypeHorizontal => {
                                    (ci.number, AxisData{ axis_type: AxisType::ScrollHorizontal(ci.increment), value: values[ci.number as usize] })
                                },
                                _ => continue
                            }
                        },
                        _ => continue
                    };

                    let mut devices = self.devices.borrow_mut();
                    let dev_info = devices.entry(dev.deviceid).or_insert_with(|| Default::default());
                    dev_info.axis_info.insert(ax_num, ax_data);
                    dev_info.has_scroll = has_scroll;
                    // store the highest axis id we need to read from events
                    if ax_num + 1 > dev_info.num_axis
                    {
                        dev_info.num_axis = ax_num + 1;
                    }
                }
            }
        }

        unsafe{ xinput2::XIFreeDeviceInfo(devices_ptr); }
    }

    fn reload_scroll_values(&self)
    {
        let scroll_devs: Vec<_> = self.devices.borrow().iter()
            .filter(|&(_, info)| info.has_scroll)
            .map(|(&dev, _)| dev).collect();

        for dev_id in scroll_devs
        {
            self.load_axis_info(dev_id);
        }
    }

    fn scancode_to_key(&self, keycode: xlib::KeyCode) -> Key
    {
        let keysym = unsafe{ xlib::XKeycodeToKeysym(self.handle, keycode, 0) };
        convert_keysym(keysym as u32)
    }

    fn with_windata<T, F>(&self, win: xlib::Window, f: F) -> T
        where T: Default, F: FnOnce(&WindowData) -> T
    {
        match self.win_data.borrow_mut().entry(win) {
            hash_map::Entry::Occupied(entry) => {
                match entry.get().upgrade() {
                    Some(wd) => f(&wd),
                    None => {
                        entry.remove();
                        T::default()
                    },
                }
            },
            _ => T::default()
        }
    }
}

impl Drop for XDisplay
{
    fn drop(&mut self)
    {
        unsafe{ xlib::XCloseDisplay(self.handle); }
    }
}

#[derive(Default)]
struct WindowData
{
    size: Cell<(u32, u32)>,
    pos: Cell<(i32, i32)>,
    ev_queue: RefCell<VecDeque<Event>>,
}

pub struct XWindow<'a>
{
    display: &'a XDisplay,
    handle: xlib::Window,
    surface: glengine::Surface<'a>,
    data: Rc<WindowData>,
}

impl<'a> XWindow<'a>
{
    fn new(display: &'a XDisplay, width: u32, height: u32) -> Result<Self, &'static str>
    {
        let win_id = unsafe {
            let screen_num = xlib::XDefaultScreen(display.handle);
            let root_win = xlib::XRootWindow(display.handle, screen_num);
            //let black_pixel = xlib::XBlackPixel(display.handle, screen_num);

            let mut win_attr = xlib::XSetWindowAttributes{
                //background_pixel: black_pixel,
                event_mask: xlib::KeyPressMask |
                            xlib::KeyReleaseMask |
                            xlib::EnterWindowMask |
                            xlib::LeaveWindowMask |
                            xlib::ExposureMask |
                            xlib::StructureNotifyMask |
                            xlib::FocusChangeMask,
                .. mem::zeroed()
            };

            let win_id = xlib::XCreateWindow(display.handle,
                root_win,       // parent
                0, 0,           // x, y
                width, height,
                0,              // border width
                xlib::CopyFromParent,       // depth
                xlib::InputOutput as u32,   // input class
                ptr::null_mut(),            // visual
                xlib::CWBackPixel | xlib::CWEventMask, // value mask
                &mut win_attr);

            // suscribe to WM close event
            let mut protocols = [display.atoms.wm_delete_window];
            if xlib::XSetWMProtocols(display.handle, win_id, &mut protocols[0], protocols.len() as i32) == xlib::False
            {
                xlib::XDestroyWindow(display.handle, win_id);
                return Err("can't set WM protocols");
            }

            // init XInput events
            let mut mask = [0];
            xinput2::XISetMask(&mut mask, xinput2::XI_ButtonPress);
            xinput2::XISetMask(&mut mask, xinput2::XI_ButtonRelease);
            xinput2::XISetMask(&mut mask, xinput2::XI_Motion);

            let mut input_event_mask = xinput2::XIEventMask{
                deviceid: xinput2::XIAllMasterDevices,
                mask_len: mask.len() as i32,
                mask: mask.as_mut_ptr(),
            };

            if xinput2::XISelectEvents(display.handle, win_id, &mut input_event_mask, 1) != xlib::Success as i32
            {
                xlib::XDestroyWindow(display.handle, win_id);
                return Err("Failed to select XInput2 events")
            }

            win_id
        };

        let surface = try!(display.engine.create_window_surface(win_id as _));

        let data = Default::default();
        display.win_data.borrow_mut().insert(win_id, Rc::downgrade(&data));

        Ok(XWindow{
            display: display,
            handle: win_id,
            surface: surface,
            data: data,
        })
    }

    pub fn set_title(&self, title: &str)
    {
        let cs = CString::new(title).unwrap();  //TODO: dont unwrap
        unsafe{ xlib::XStoreName(self.display.handle, self.handle, cs.as_ptr()); }
    }

    pub fn show(&self)
    {
        unsafe{ xlib::XMapWindow(self.display.handle, self.handle); }
    }

    pub fn get_size(&self) -> (u32, u32)
    {
        self.data.size.get()
    }

    pub fn get_position(&self) -> (i32, i32)
    {
        self.data.pos.get()
    }

    pub fn consume_event(&self) -> Option<Event>
    {
        self.data.ev_queue.borrow_mut().pop_front()
    }

    pub fn draw(&self) -> glengine::DrawContext
    {
        self.display.engine.begin_draw(&self.surface, self.data.size.get())
    }
}

impl<'a> Drop for XWindow<'a>
{
    fn drop(&mut self)
    {
        unsafe
        {
            xlib::XDestroyWindow(self.display.handle, self.handle);
        }
    }
}
