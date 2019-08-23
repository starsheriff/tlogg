use structopt::StructOpt;
use xdg;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(short = "d", long = "duration")]
    /// Number of hours spent
    ///
    /// The number of hours spent on the current entry.
    hours: f64,

    #[structopt(short = "m", long = "message")]
    /// Description of the task for the current entry.
    ///
    /// Should be kept short. Preferrably less than 70 characters.
    description: String,

    #[structopt(short = "p", long = "project")]
    /// Name of the project to log the hours.
    ///
    /// The name of the project to log hours on. If not provided, defaults
    /// back to the last used project.
    project : Option<String>,

    #[structopt(name = "verbose", short, long)]
    /// Verbose output
    ///
    /// Prints debug information.
    flag_verbose: bool,
}

fn main() {
    let args = Args::from_args();

    // find correct database
    let xdg_dirs = xdg::BaseDirectories::with_prefix("tlogg").unwrap();

    let db_path = xdg_dirs
        .place_data_file("tlogg.sqlite")
        .expect("cannot create database");

    if args.flag_verbose {
        println!("opening db: {}", db_path.as_path().display());
    }

    println!("{:?}", args);
    println!("{:?}", xdg_dirs);
}
