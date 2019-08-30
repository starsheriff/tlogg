use rusqlite::{params, Connection, Result};
use std::path::Path;
//use structopt::clap::Shell;
use structopt::StructOpt;
use xdg;

/// currently used version of database schema
pub const DB_VERSION: usize = 1;

const SQL_1: &str = include_str!("../sql/01_init.sql");

#[derive(Debug, StructOpt)]
struct Tlogg {
    #[structopt(name = "verbose", short, long)]
    /// Verbose output
    ///
    /// Prints debug information.
    flag_verbose: bool,

    #[structopt(subcommand)]
    command: Commands,
}


#[derive(Debug, StructOpt)]
enum Commands {
    #[structopt(name = "add")]
    /// Add a new time log entry to an existing project.
    ///
    /// The project may be explicitely specified by its name. If not provided, the
    /// last project is used to add the entry.
    Add(Add),

    #[structopt(name = "rm")]
    /// Remove a time log entry.
    ///
    /// Remove an entry from the log.
    Rm(Remove),

    #[structopt(name = "add-project")]
    /// Add a new project to log hours on
    AddProject(AddProject),

    #[structopt(name = "rm-project")]
    /// Add a new project to log hours on
    RemoveProject(RemoveProject),

    #[structopt(name = "print")]
    /// Export the logs.
    Print(Print),
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
struct Remove {
    #[structopt()]
    /// Unique id of the entry
    id: usize,
}

#[derive(Debug, StructOpt)]
struct AddProject {
    #[structopt(long, short)]
    /// Name of the new project
    ///
    /// Should be short and concise.
    name: String,

    #[structopt(long, short)]
    /// Description of the new project
    description: String,
}

#[derive(Debug, StructOpt)]
struct RemoveProject {
    #[structopt()]
    /// Unique id of the entry
    name: String,
}


#[derive(Debug, StructOpt)]
struct Print {
    #[structopt(subcommand)]
    format: Format,
    #[structopt(name = "from", long, short)]
    from: String,
}

#[derive(Debug, StructOpt)]
enum Format{
    #[structopt(name = "markdown")]
    /// Print as a human readable markdown file.
    Markdown,
    #[structopt(name = "csv")]
    /// Print as machine readable csv file.
    CSV,
}


/// new_connection returns a new connection to the database at the given
/// location.
///
/// The connection is properly initialized and all pragmas, e.g. `foreign_keys`,
/// are set.
pub fn new_connection(p: &Path) -> Result<Connection> {
    let conn = Connection::open(p)?;
    // foreign key support must be explicitely enabled _per connection_
    conn.execute("PRAGMA foreign_keys = ON", rusqlite::NO_PARAMS)?;

    Ok(conn)
}

#[derive(Debug)]
pub struct Project {
    name: String,
    description: String,
}

impl From<&AddProject> for Project {
    fn from(args: &AddProject) -> Project {
        Project{
            name: args.name.clone(),
            description: args.description.clone(),
        }
    }
}

#[derive(Debug)]
pub struct TloggEntry {
    id: Option<usize>,
    duration: f64,
    message: String,
    project: String,
}

impl From<&Add> for TloggEntry {

    fn from(add_args: &Add) -> TloggEntry {
        let project = match &add_args.project {
            Some(projectName) => projectName.clone(),
            None => String::new(),
        };

        TloggEntry{
            id: None,
            duration: add_args.hours,
            message: add_args.description.clone(),
            project,
        }
    }

}


pub fn user_version(conn: &Connection) -> rusqlite::Result<usize> {
    let x: rusqlite::Result<i64> =
        match conn.query_row("PRAGMA user_version", rusqlite::NO_PARAMS, |row| row.get(0)) {
        Ok(version) => Ok(version),
        Err(e) => Err(e),
    };

    match x {
        Ok(version) => Ok(version as usize),
        Err(e) => Err(e),
    }
}

pub fn migrate(conn: &Connection) -> rusqlite::Result<usize> {
    let current_version: usize= user_version(conn)?;

    if current_version == DB_VERSION {
        return Ok(DB_VERSION)
    }

    if current_version < 1 {
        // TODO: transaction
        println!("running migration to version 1");
        conn.execute(&format!("PRAGMA user_version = {}", 1), rusqlite::NO_PARAMS).unwrap();
        conn.execute_batch(SQL_1)?;
    }
    // if new migrations are required they have to be added here

    Ok(DB_VERSION)
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

    // check version and migrate if necessary
    let user_version = user_version(&conn).unwrap();

    if args.flag_verbose {
        println!("Database schema version: {}", user_version);
    }
    if user_version < DB_VERSION {
        println!("schema migration required. {}<{}", user_version, DB_VERSION);

        migrate(&conn).unwrap();
    }

    // run commands
    commands(args, conn).unwrap();
}


fn commands(args: Tlogg, conn: rusqlite::Connection) -> Result<()> {
    match &args.command {
        Commands::Add(add_args) => {
            cmd_add(&args, &add_args, conn)
        },
        Commands::AddProject(add_project_args) => {
            cmd_add_project(&args, &add_project_args, conn)
        },
        _ => {
            panic!("not implemented");
        },

    }
}

fn cmd_add(args: &Tlogg, add_args: &Add, conn: rusqlite::Connection) -> Result<()>{
    if args.flag_verbose {
        println!("called command add");
    }

    let entry = TloggEntry::from(add_args);

    add_entry(&conn,  entry)
}

fn cmd_add_project(args: &Tlogg, add_project_args: &AddProject, conn: Connection) -> Result<()> {
    if args.flag_verbose {
        println!("called command add-project")
    }

    let project = Project::from(add_project_args);

    add_project(&conn, &project)
}

pub fn add_entry(conn: &Connection, entry: TloggEntry) -> Result<()> {
    let mut stmt = conn.prepare("INSERT INTO log_entries (description, hours, project) VALUES (?,?,?);")?;
    stmt.execute(params![entry.message, entry.duration, entry.project])?;
    Ok(())
}

pub fn add_project(conn: &Connection, p: &Project) -> Result<()> {
    let mut stmt = conn.prepare("INSERT INTO projects (name, description) VALUES (?, ?);")?;
    stmt.execute(&[&p.name, &p.description])?;
    Ok(())
}

pub fn list_projects(conn: &Connection) -> Option<Vec<Project>> {
    let mut stmt = conn.prepare("SELECT name, description FROM projects").
        expect("could not prepare statement");
    let projects = stmt.
        query_map(rusqlite::NO_PARAMS, |row| Ok(Project{
            name: row.get(0)?,
            description: row.get(1)?,
        })).expect("somethings wrong").
    map(|x| x.unwrap()).collect();

    println!("{:?}", projects);

    Some(projects)
}



#[cfg(test)]
mod test {
    use super::*;

    struct TestCase {
        db: Connection,
    }

    impl TestCase {
        fn new() -> TestCase {
            TestCase{
                db: new_connection(Path::new(":memory:")).expect("could not open db"),
            }
        }

        fn migrate(self) -> Self {
            migrate(&self.db).expect("could not migrate");
            self
        }

        fn when_add_project(self, p: &Project) -> Self {
            add_project(&self.db, p).expect("could not add project"); self
        }

        fn when_list_projects(self) -> Self {
            list_projects(&self.db);
            self
        }

    }

    #[test]
    fn test_create_bucket() {
        TestCase::new().
            migrate().
            when_add_project(&Project{
                name: "testproject".to_string(),
                description: "just testing".to_string(),
            });
            //when_add_bucket("myBucket").
            //then_bucket_exists("myBucket");
    }

    #[test]
    fn test_list_projects() {
        TestCase::new().
            migrate().
            when_add_project(&Project{
                name: "testproject".to_string(),
                description: "just testing".to_string(),
            }).
            when_list_projects();
    }

}
