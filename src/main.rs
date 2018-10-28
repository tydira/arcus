extern crate ansi_term;
extern crate clap;
extern crate hex;
extern crate unicode_segmentation;

use std::io;
use std::process;
use std::string::ToString;

use ansi_term::Colour::RGB;
use ansi_term::{ANSIString, ANSIStrings};
use clap::{App, Arg};
use unicode_segmentation::UnicodeSegmentation;

// Hi there,
// I'm still new to Rust, so please suggest improvements.
// - Justin (<3)

type Colors = Vec<Vec<u8>>;

fn decode_colors(colors: Vec<&str>) -> Colors {
    colors
        .iter()
        .map(|color| {
            hex::decode(color.replace("#", "").as_bytes()).unwrap_or_else(|_| {
                eprintln!("error: Invalid color '{}'", color);
                process::exit(2);
            })
        })
        .collect()
}

fn calculate_delta(from: Vec<u8>, to: Vec<u8>) -> Vec<i16> {
    from.iter()
        .zip(to.iter())
        .map(|(&l, &r)| r as i16 - l as i16)
        .collect::<Vec<i16>>()
}

fn decorate_string<'a>(input: String, colors: Colors) -> String {
    let output: &mut Vec<ANSIString> = &mut vec![];

    let graphemes: Vec<&str> = input.graphemes(true).collect();
    let total_colors: &mut Vec<ansi_term::Color> = &mut vec![];

    let total_steps = graphemes.len() as f64;
    let colors_length = colors.len();
    let distance = (total_steps / (colors_length as f64 - 1.0)).floor() as i16;

    for (i, color) in colors.iter().enumerate() {
        if i + 1 < colors_length {
            let d = calculate_delta(color.to_vec(), colors[i + 1].to_vec());

            let r_step = d[0] / distance;
            let g_step = d[1] / distance;
            let b_step = d[2] / distance;

            for j in 0..distance {
                total_colors.push(RGB(
                    color[0] + (r_step * j as i16) as u8,
                    color[1] + (g_step * j as i16) as u8,
                    color[2] + (b_step * j as i16) as u8,
                ));
            }
        }
    }

    for (i, c) in total_colors.iter().enumerate() {
        output.push(c.paint(graphemes[i]));
    }

    return ANSIStrings(output).to_string();
}

fn main() {
    let matches = App::new("arcus")
        .version("0.1.0")
        .version_short("v")
        .author("Justin Krueger <justin@kroo.gs>")
        .about("Decorates stdin with gradients made of 24-bit ANSI color codes.")
        .arg(
            Arg::with_name("colors")
                .help("Hexadecimal colors to interpolate between")
                .required(true)
                .multiple(true)
                .index(1),
        )
        .get_matches();

    let colors = decode_colors(matches.values_of("colors").unwrap().collect());
    if colors.len() < 2 {
        eprintln!("error: Requires at least 2 color arguments");
        process::exit(1);
    }

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            print!("{}", decorate_string(input, colors));
        }
        Err(error) => {
            println!("error: {}", error);
            process::exit(3);
        }
    }
}
