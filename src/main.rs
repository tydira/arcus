extern crate ansi_term;
extern crate clap;
extern crate hex;
extern crate unicode_segmentation;

use std::io;
use std::process::exit;

use ansi_term::Colour::RGB;
use clap::{App, Arg};
use unicode_segmentation::UnicodeSegmentation;

type Color = Vec<u8>;
type Colors = Vec<Color>;
type GraphemeColors<'a> = Vec<(&'a str, ansi_term::Color)>;

fn decode_colors(colors: Vec<&str>) -> Colors {
    colors
        .iter()
        .map(|color| {
            hex::decode(color.replace("#", "").as_bytes()).unwrap_or_else(|_| {
                eprintln!("error: Invalid color '{}'", color);
                exit(1);
            })
        })
        .collect()
}

fn calculate_deltas(from: &Vec<u8>, to: &Vec<u8>) -> Vec<i16> {
    from.iter()
        .zip(to.iter())
        .map(|(&f, &t)| t as i16 - f as i16)
        .collect::<Vec<_>>()
}

fn interpolate(input: &str, colors: Colors) -> GraphemeColors {
    let graphemes = input.graphemes(true);
    let grapheme_count = graphemes.clone().collect::<Vec<_>>().len();

    let color_count = colors.len() - 1;
    let color_distance = grapheme_count / color_count;
    let color_remainder = grapheme_count % color_count;

    let interpolated = colors
        .iter()
        .enumerate()
        .map(|(i, color)| {
            if i < color_count {
                let steps = calculate_deltas(color, &colors[i + 1])
                    .iter()
                    .map(|&d| d / color_distance as i16)
                    .collect::<Vec<_>>();

                (0..color_distance)
                    .map(|i| {
                        let i = i as i16;
                        RGB(
                            color[0] + (steps[0] * i) as u8,
                            color[1] + (steps[1] * i) as u8,
                            color[2] + (steps[2] * i) as u8,
                        )
                    })
                    .collect::<Vec<_>>()
            } else {
                let last = colors.last().unwrap().to_vec();
                (0..color_remainder)
                    .map(|_| RGB(last[0], last[1], last[2]))
                    .collect::<Vec<_>>()
            }
        })
        .flatten()
        .collect::<Vec<_>>();

    graphemes.zip(interpolated).collect::<Vec<_>>()
}

fn output(input: GraphemeColors) {
    input.iter().for_each(|(grapheme, color)| {
        print!("{}", color.paint(*grapheme));
    })
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
        exit(2);
    }

    let input = &mut String::new();
    match io::stdin().read_line(input) {
        Ok(_) => output(interpolate(input, colors)),
        Err(error) => {
            eprintln!("error: {}", error);
            exit(3);
        }
    }
}
