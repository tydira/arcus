extern crate ansi_term;
extern crate clap;
extern crate hex;
extern crate unicode_segmentation;

use std::io::stdin;
use std::iter::repeat;
use std::process::exit;

use ansi_term::Colour::RGB;
use clap::{App, Arg};
use unicode_segmentation::UnicodeSegmentation;

type Color = Vec<u8>;
type Colors = Vec<Color>;

fn decode(colors: Vec<&str>) -> Colors {
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

fn calculate_delta<'a>(from: &'a Color, to: &'a Color) -> impl Iterator<Item = i16> + 'a {
    from.iter()
        .zip(to.iter())
        .map(|(&f, &t)| t as i16 - f as i16)
}

fn interpolate(colors: Colors, length: usize) -> Colors {
    let distance = colors.len() - 1;
    let segment_size = length / distance;
    let segment_remainder = length % distance;

    colors
        .iter()
        .enumerate()
        .map(|(i, color)| -> Vec<_> {
            if i < distance {
                let deltas: Vec<_> = calculate_delta(color, &colors[i + 1])
                    .map(|d| d / segment_size as i16)
                    .collect();

                (0..segment_size)
                    .map(|j| {
                        (0..3)
                            .map(|n| color[n] + (deltas[n] * j as i16) as u8)
                            .collect()
                    })
                    .collect()
            } else {
                let last = colors.last().unwrap();
                repeat(last.to_vec()).take(segment_remainder).collect()
            }
        })
        .flatten()
        .collect()
}

fn output(graphemes: Vec<&str>, colors: Colors) {
    graphemes.iter().enumerate().for_each(|(i, grapheme)| {
        print!(
            "{}",
            RGB(colors[i][0], colors[i][1], colors[i][2]).paint(*grapheme)
        );
    })
}

fn main() {
    let matches = App::new("arcus")
        .version("0.2.0")
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

    let colors = decode(matches.values_of("colors").unwrap().collect());
    if colors.len() < 2 {
        eprintln!("error: Requires at least 2 color arguments");
        exit(2);
    }

    let input = &mut String::new();
    match stdin().read_line(input) {
        Ok(_) => {
            let graphemes: Vec<_> = input.graphemes(true).collect();
            let grapheme_count = graphemes.len();
            output(graphemes, interpolate(colors, grapheme_count));
        }
        Err(error) => {
            eprintln!("error: {}", error);
            exit(3);
        }
    }
}
