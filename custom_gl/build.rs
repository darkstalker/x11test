extern crate gl_generator;

use gl_generator::{Registry, Api, Profile, Fallbacks};
use std::env;
use std::fs::File;
use std::path::Path;

fn main()
{
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("gl_bindings.rs");
    let mut file = File::create(&dest).unwrap();

    Registry::new(Api::Gles2, (2, 0), Profile::Core, Fallbacks::All, [
        ])
        .write_bindings(gl_generator::StructGenerator, &mut file)
        .unwrap();
}
