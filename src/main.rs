use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Args {
    hours: f64,
    description: String,
    project : String,
}

fn main() {
    let args = Args::from_args();

    println!("{:?}", args);
}
