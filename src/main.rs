use clap::{Arg, App, crate_version, crate_authors, crate_description};

fn main() {
    let matches = App::new("topfs")
    .version(crate_version!())
    .author(crate_authors!())
    .about(crate_description!())
    
    .get_matches();
}
