[![NPM package](https://img.shields.io/crates/v/hsml.svg)](https://crates.io/crates/hsml)
[![Downloads](https://img.shields.io/crates/d/hsml.svg)](https://crates.io/crates/hsml)
[![NPM package](https://img.shields.io/npm/v/hsml.svg)](https://www.npmjs.com/package/hsml)
[![Downloads](https://img.shields.io/npm/dt/hsml.svg)](https://www.npmjs.com/package/hsml)
[![Build Status](https://github.com/hsml-lab/hsml/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/hsml-lab/hsml/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/github/license/hsml-lab/hsml.svg)](https://github.com/hsml-lab/hsml/blob/main/LICENSE)
[![Donate: PayPal](https://img.shields.io/badge/Donate-PayPal-blue.svg)](https://www.paypal.com/donate?hosted_button_id=L7GY729FBKTZY)

# UNDER CONSTRUCTION

Right now there is no stable version of `hsml` available. I'm just working on it.

<img src="https://chronicle-brightspot.s3.amazonaws.com/6a/c4/00e4ab3143f7e0cf4d9fd33aa00b/constructocat2.jpg" width="400px" />

# HSML - Hyper Short Markup Language

`hsml` is a hyper short markup language that is inspired by [pug](https://pugjs.org) (aka jade).

## What is it?

- `hsml` is written in [Rust](https://www.rust-lang.org) and compiles to HTML.
- There will be a binary that can take CLI arguments to compile a `.hsml` file to a `.html` file, but also there will be some other arguments to e.g. format a `.hsml` file.
- There will be also a library that can parse a `.hsml` file and return an AST for it. It is planned that this AST can be used in the JS ecosystem as well, so tools like ESLint and Prettier can work with it.
- There are two major differences between `pug` and `hsml`
  - `hsml` will support TailwindCSS and similar CSS frameworks out of the box, even with arbitrary values like `.bg-[#1da1f2]` or `lg:[&:nth-child(3)]:hover:underline`
  - `hsml` will **not** support template engine syntax. It is _just_ an HTML preprocessor.

## Why doing it?

- I want to learn Rust
- I use `pug` for my projects but sadly `pug`'s goal mismatches my preferences and comes with a lot of overhead I don't need
