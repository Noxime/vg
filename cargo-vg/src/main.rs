use cargo_vg::*;
use structopt::StructOpt;

fn main() {
    let Args::Vg(opts) = Args::from_args();

    run(opts)
}
