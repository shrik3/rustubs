//! a simple shell...
use crate::arch::x86_64::misc::delay;
use crate::fs::*;
use crate::io::{back_space, read_key};
use crate::kthread::KThread;
use crate::proc::task::Task;
use alloc::vec::Vec;
use core::str;
pub struct Kshell {}

impl KThread for Kshell {
	fn entry() -> ! {
		let t = Task::current().unwrap();
		let files = ls();
		event_loop(&files);
	}
}

fn event_loop(files: &Vec<File>) -> ! {
	let mut input_buffer = Vec::<u8>::new();
	print!("$ ");
	loop {
		let k = read_key().get_ascii();
		match k {
			0x8 => {
				// backspace
				if let Some(_) = input_buffer.pop() {
					back_space();
				}
			}
			0xa => {
				// enter
				println!();
				let cmd: &str = str::from_utf8(&input_buffer).unwrap();
				handle(cmd, files);
				input_buffer.clear();
				print!("$ ");
			}
			_ => {
				print!("{}", k as char);
				input_buffer.push(k)
			}
		}
	}
}

fn handle(cmd: &str, files: &Vec<File>) {
	let tokens: Vec<&str> = cmd.split(' ').collect();
	match tokens[0] {
		"ls" => files.iter().for_each(|f| println!("{}", f.hdr.name())),
		"cat" => {
			if tokens.len() < 2 {
				println!("need file name");
				return;
			}
			let file_name = tokens[1];
			if let Some(file) = files.iter().find(|f| f.hdr.name() == file_name) {
				cat(file)
			} else {
				println!("{}: no such file or directory", file_name,);
			}
		}
		_ => {
			println!("invalid command: {}", cmd);
		}
	}
}
