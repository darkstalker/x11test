extern crate x11test;
use x11test::{XDisplay, Event, EvState, Key};

fn main()
{
    let display = XDisplay::new().unwrap();

    let window = display.create_window(300, 300).unwrap();
    window.set_title("main");
    window.show();

    let mut others = Vec::new();
    let mut n = 1;

    'main: loop
    {
        // get and store event
        display.wait_event();

        // pull events for this window
        while let Some(ev) = window.consume_event()
        {
            match ev {
                Event::CloseButton => break 'main,    // WM delete event
                Event::Keyboard(EvState::Pressed, Key::Insert) => {
                    let win = display.create_window(150, 150).unwrap();
                    win.set_title("child");
                    win.show();
                    others.push((n, win));
                    n += 1;
                },
                /*Event::Keyboard(KeyState::Pressed, 9) => {
                    x11test::stuff::enumerate_devices(display.handle);  // debug stuff
                }*/
                Event::Keyboard(EvState::Pressed, Key::Unk(ks)) => {
                    println!("** keysym: {:x}", ks);
                },
                _ => println!(">> main: {:?}", ev)
            }
        }

        // handling a bunch of windows
        others.retain(|&(id, ref win)| {
            while let Some(ev) = win.consume_event()
            {
                println!(">> child {}: {:?}", id, ev);
                if let Event::CloseButton = ev
                {
                    // dropping closes the window
                    println!(">> closing {}", id);
                    return false
                }
            }
            true
        });
    }
}