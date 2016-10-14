extern crate gl_generator;

use gl_generator::{Registry, Api, Profile, Fallbacks};
use std::env;
use std::fs::File;
use std::path::Path;

fn main()
{
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("egl_bindings.rs");
    let mut file = File::create(&dest).unwrap();

    Registry::new(Api::Egl, (1, 5), Profile::Core, Fallbacks::All, [
        ])
        .write_bindings(gl_generator::StaticGenerator, &mut file)
        .unwrap();
}
