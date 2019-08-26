use rusqlite::Connection;
use std::path::Path;
use structopt::clap::Shell;
use structopt::StructOpt;
use xdg;

#[derive(Debug, StructOpt)]
struct Tlogg {
    #[structopt(name = "verbose", short, long)]
    /// Verbose output
    ///
    /// Prints debug information.
    flag_verbose: bool,

    #[structopt(subcommand)]
    Command: Commands,
}

#[derive(Debug, StructOpt)]
enum Commands {
    #[structopt(name = "add")]
    /// Add a new time log entry to an existing project.
    ///
    /// The project may be explicitely specified by its name. If not provided, the
    /// last project is used to add the entry.
    Add(Add),

    #[structopt(name = "add-project")]
    /// Add a new project to log hours on
    ///
    AddProject(AddProject),
}

#[derive(Debug, StructOpt)]
struct Add {
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
    project: Option<String>,
}

#[derive(Debug, StructOpt)]
struct AddProject {
    #[structopt(name = "name", long, short)]
    /// Name of the new project
    ///
    /// Should be short and concise.
    name: String,
}


/// new_connection returns a new connection to the database at the given
/// location.
///
/// The connection is properly initialized and all pragmas, e.g. `foreign_keys`,
/// are set.
pub fn new_connection(p: &Path) -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open(p)?;
    // foreign key support must be explicitely enabled _per connection_
    conn.execute("PRAGMA foreign_keys = ON", rusqlite::NO_PARAMS)?;

    Ok(conn)
}

fn main() {
    let args = Tlogg::from_args();

    // find correct database
    let xdg_dirs = xdg::BaseDirectories::with_prefix("tlogg").unwrap();

    //Args::clap().gen_completions(env!("CARGO_PKG_NAME"), Shell::Zsh, xdg_dirs.get_config_home());
    //println!("{:?}", xdg_dirs.get_config_home());
    //Args::clap().gen_completions("testme", Shell::Zsh, xdg_dirs.get_config_home());

    let db_path = xdg_dirs
        .place_data_file("tlogg.sqlite")
        .expect("cannot create database");

    if args.flag_verbose {
        println!("opening db: {}", db_path.as_path().display());
    }

    // open connection
    let conn = new_connection(db_path.as_path()).unwrap();
    println!("{:?}", conn);

    println!("{:?}", args);
    println!("{:?}", xdg_dirs);
}
