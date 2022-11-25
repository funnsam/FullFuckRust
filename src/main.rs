use std::fs;
use std::env;
use std::str;

mod compiler;

use compiler::compile;

fn main() {
    let mut no_hlt_enabled: bool = false;

    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("\x1b[1;31m>:(\t\tSee the docs! -> https://github.com/funnsam/FullFuck/wiki/Basic-syntax\n\x1b[1;0m");
        return;
    }

    for el in args.iter() {
        let real_el = el.as_str();

        match real_el {
            "--no-hlt"      => no_hlt_enabled = true,
            &_              => (),
        }
    }

    let input_data = read_file(args[1].as_str()).unwrap();
    write_file(args[2].as_str(), compile(input_data, no_hlt_enabled));
}

fn read_file(filepath: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let data = fs::read(filepath)?;
    Ok(data)
}

fn write_file(filepath: &str, content: String) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(filepath, content.as_str())?;
    Ok(())
}