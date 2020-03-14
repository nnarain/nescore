//
// NES tools
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Mar 01 2020
//
use nescli::{Options, Command};
use clap::Clap;

fn main() {
    let opts = Options::parse();

    match opts.cmd {
        Command::Run(opts)  => nescli::run::dispatch(opts),
        Command::Info(opts) => nescli::info::dispatch(opts),
        Command::Img(opts)  => nescli::img::dispatch(opts),
    }
}
