# checkers

This repository holds my first attempt at a chess engine in my favorite
programming language: Rust. Hence the name `checkers` for "check-rs", `.rs`
being the file extension for Rust source code.

## About me

I am [a reasonable chess player](https://lichess.org/@/BenSchwanz) of about 2000
online ELO strength. In my day job, I am employed as a n R&D engineer, working
on hard- and software projects in a research institute. The logical intersection
of these two lead me to finally try writing my own engine, in the hopes of
creating something I regret i.e., can beat me.

## Features

- Bitboards for efficient move generation
- Lookup table for pseudo-legal piece moves, assuming an empty board
- Rendering an ASCII representation of the board
