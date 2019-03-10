#[macro_use]
extern crate clap;
extern crate colored;
extern crate m3u;
extern crate rand;

use std::fs;
use std::path::PathBuf;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use colored::*;
use rand::{thread_rng, Rng};

fn export(playlist: &str, output: &str, shuffle: bool) -> Result<(), std::io::Error> {
    // need to copy each file then make a new playlist with only the
    // file names without the path
    let input = PathBuf::from(playlist);
    let output = PathBuf::from(output);

    let input_filename = input.file_name().unwrap();

    let output_filename = output.join(input_filename);

    let mut output_file = std::fs::File::create(output_filename.clone())?;
    let mut writer = m3u::Writer::new(&mut output_file);

    let mut reader = m3u::Reader::open(playlist)?;
    let mut read_playlist: Vec<_> = reader.entries().map(|entry| entry.unwrap()).collect();

    println!(
        "exporting '{}' to '{}'",
        input.display(),
        output_filename.display()
    );

    if shuffle {
        thread_rng().shuffle(&mut read_playlist);
    }

    for entry in &read_playlist {
        match entry {
            m3u::Entry::Path(path) => {
                let filename = path.file_name().unwrap();

                let dest = output.join(filename);

                if !dest.exists() {
                    println!("{} {} => {}", "exporting".green().bold(), path.display(), dest.display());
                    fs::copy(path, dest)?;
                } else {
                    println!("   {} {}", "exists".yellow().bold(), path.display());
                }

                let entry = m3u::path_entry(filename);
                writer.write_entry(&entry)?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn shuffle(matches: &ArgMatches) {}

fn main() {
    let matches = App::new("playlist-exporter")
        .about("export m3u playlists and their referenced media files")
        .author(crate_authors!())
        .version(crate_version!())
        .global_setting(AppSettings::ColorAuto)
        .global_setting(AppSettings::ColoredHelp)
        .subcommand(
            SubCommand::with_name("export")
                .about("export a playlist with ereferenced files")
                .arg(
                    Arg::with_name("playlist")
                        .help("the input playlist")
                        .index(1)
                        .required(true),
                ).arg(
                    Arg::with_name("output")
                        .help("the output directory")
                        .takes_value(true)
                        .short("o")
                        .long("output")
                        .required(true),
                ).arg(
                    Arg::with_name("shuffle")
                        .help("shuffle the output playlist")
                        .takes_value(false)
                        .short("s")
                        .long("shuffle")
                        .required(false),
                ),
        ).subcommand(
            SubCommand::with_name("shuffle")
                .about("shuffle an existing playlist")
                .arg(
                    Arg::with_name("playlist")
                        .help("the input playlist")
                        .index(1)
                        .required(true),
                ),
        ).get_matches();

    match matches.subcommand() {
        ("export", Some(export_matches)) => {
            export(
                export_matches.value_of("playlist").unwrap(),
                export_matches.value_of("output").unwrap(),
                export_matches.is_present("shuffle")
            ).unwrap()
        }
        ("shuffle", Some(shuffle_matches)) => shuffle(shuffle_matches),
        ("", None) => {}
        _ => unreachable!(),
    }
}
