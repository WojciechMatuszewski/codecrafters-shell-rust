#![allow(deprecated)]

use std::io;

use command::Command;
use executable::{PathFinder, Runner};
use prompt::{ConsolePrompter, Prompter};

mod command;
mod executable;
mod prompt;

fn main() -> anyhow::Result<()> {
    let reader = io::stdin().lock();
    let writer = io::stdout();
    let mut prompter = ConsolePrompter::new(reader, writer);

    let finder = PathFinder::new();
    let runner = Runner::new();

    loop {
        prompter.prompt("$ ")?;

        let input = prompter.read()?;

        let command = input.parse::<Command>()?;
        command.run(&mut prompter, &finder, &runner)?;
    }
}
