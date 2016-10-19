extern crate x11test;
extern crate rand;
use x11test::{XDisplay, Event, EvState, Key, Button};
use rand::Rng;

fn main()
{
    let display = XDisplay::new().unwrap();

    let window = display.create_window(640, 480).unwrap();
    window.set_title("main");
    window.show();

    let mut others = Vec::new();
    let mut n = 1;
    let mut mdown = false;

    'main: loop
    {
        // get and store event
        display.wait_event();

        // pull events for this window
        while let Some(ev) = window.consume_event()
        {
            match ev
            {
                Event::CloseButton | Event::Keyboard(EvState::Pressed, Key::Escape) => break 'main,
                Event::Keyboard(EvState::Pressed, Key::Insert) => {
                    let win = display.create_window(150, 150).unwrap();
                    win.set_title("child");
                    win.show();
                    others.push((n, win));
                    n += 1;
                },
                Event::Keyboard(EvState::Pressed, Key::Q) => {
                    let (w, h) = window.get_size();
                    let mut rng = rand::thread_rng();
                    let mut ctx = window.draw();
                    for _ in 0..1000
                    {
                        ctx.draw_rect(rng.gen_range(0, w as i16), rng.gen_range(0, h as i16),
                            rng.gen_range(10, 200), rng.gen_range(10, 200),
                            rng.gen());
                    }
                }
                Event::Keyboard(EvState::Pressed, Key::W) => {
                    let (w, h) = window.get_size();
                    let mut rng = rand::thread_rng();
                    let mut ctx = window.draw();
                    for _ in 0..1000
                    {
                        ctx.draw_line(rng.gen_range(0, w as i16), rng.gen_range(0, h as i16),
                            rng.gen_range(0, w as i16), rng.gen_range(0, h as i16),
                            rng.gen());
                    }
                }
                Event::Keyboard(EvState::Pressed, Key::E) => {
                    let (w, h) = window.get_size();
                    let mut rng = rand::thread_rng();
                    let mut ctx = window.draw();
                    for _ in 0..1000
                    {
                        ctx.draw_triangle(rng.gen_range(0, w as i16), rng.gen_range(0, h as i16),
                            rng.gen_range(0, w as i16), rng.gen_range(0, h as i16),
                            rng.gen_range(0, w as i16), rng.gen_range(0, h as i16),
                            rng.gen());
                    }
                }
                Event::Keyboard(EvState::Pressed, Key::R) => {
                    let (w, h) = window.get_size();
                    let mut rng = rand::thread_rng();
                    let mut ctx = window.draw();
                    for _ in 0..1000
                    {
                        ctx.draw_point(rng.gen_range(0, w as i16), rng.gen_range(0, h as i16), rng.gen());
                    }
                }
                Event::Keyboard(EvState::Pressed, Key::Unk(ks)) => {
                    println!("** keysym: {:x}", ks);
                },
                Event::Redraw => {
                    let mut ctx = window.draw();
                    ctx.clear([0.1, 0.1, 0.1, 1.0]);
                }
                Event::MouseButton(EvState::Pressed, Button::Left, (x, y)) => {
                    mdown = true;
                    let mut ctx = window.draw();
                    ctx.draw_rect(x as i16 - 5, y as i16 - 5, 10, 10, [1.0, 0.0, 0.0, 1.0]);
                },
                Event::MouseButton(EvState::Released, Button::Left, _) => {
                    mdown = false;
                },
                Event::MouseMoved(x, y) if mdown => {
                    let mut ctx = window.draw();
                    ctx.draw_rect(x as i16 - 5, y as i16 - 5, 10, 10, [1.0, 0.0, 0.0, 1.0]);
                },
                //_ => println!(">> main: {:?}", ev)
                _ => ()
            }
        }

        // handling a bunch of windows
        others.retain(|&(id, ref win)| {
            while let Some(ev) = win.consume_event()
            {
                match ev
                {
                    Event::Redraw => {
                        let mut ctx = win.draw();
                        ctx.clear([0.1, 0.1, 0.1, 1.0]);
                        ctx.draw_triangle(10, 10, 100, 20, 50, 100, [1.0, 1.0, 0.0, 1.0]);
                    }
                    Event::CloseButton => {
                        // dropping closes the window
                        println!(">> closing {}", id);
                        return false
                    }
                    _ => println!(">> child {}: {:?}", id, ev)
                }
            }
            true
        });
    }
}
