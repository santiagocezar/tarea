# TAREA

_The universe is infinite._ That means there's also infinite to-do list apps, and most of them were made in Earth, probably. Well, this is one of them.

## Why?

The best way to not forget to read my to-do list is by putting it in the place I use most. I didn't want to clutter my phone homescreen with a big to-do list widget, so instead I've decided to clutter my terminal instead!

I also wanted to run this app every time I open a terminal, so it had to be quick and simple. That's why I wrote it in Rust

## Didn't an app already do this?

Not sure, I didn't really check and I wanted to make my own anyways.

## How do I run this?

Clone this repo, then run `cargo install --path .`. That should downlaod and compile all the dependencies.

After that run `echo tarea >> .bashrc` (for zsh and fish, use `.zshrc` and `~/.config/fish/config.fish` respectively). Now every time you open a terminal it'll show your to-do list.

- To add a task run `tarea add [your task]`
- To list tasks run `tarea list` or just `tarea`
- To mark a task as done `tarea done [task number]`

Run `tarea help` for a complete list of commands.


