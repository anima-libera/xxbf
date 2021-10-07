mod graph;
mod astraw;
mod astsoup;
mod ctranspiler;
mod parser;
mod vm;

#[derive(Debug)]
enum WhatToDo {
	Interpret {
		input: Option<String>,
	},
	Compile {
		target: CompileTarget,
		dst_file_path: Option<String>,
	},
}

#[derive(Debug)]
enum CompileTarget {
	C,
}

#[derive(Debug)]
enum SrcSettings {
	Src(String),
	FilePath(String),
	None,
}

#[derive(Debug)]
struct Settings {
	path: Option<String>,
	help: bool,
	verbose: bool,
	src: SrcSettings,
	optimize: bool,
	what_to_do: WhatToDo,
}

impl Settings {
	fn from_cmdline_args() -> Settings {
		let mut args = std::env::args();
		let mut settings = Settings {
			path: args.next(),
			help: false,
			verbose: false,
			src: SrcSettings::None,
			optimize: true,
			what_to_do: WhatToDo::Interpret { input: None },
		};
		while let Some(arg) = args.next() {
			if arg == "-h" || arg == "--help" {
				settings.help = true;
			} else if arg == "-v" || arg == "--verbose" {
				settings.verbose = true;
			} else if arg == "-s" || arg == "--src" {
				settings.src = SrcSettings::Src(args.next().unwrap());
			} else if arg == "-f" || arg == "--src-file" {
				settings.src = SrcSettings::FilePath(args.next().unwrap());
			} else if arg == "-O0" || arg == "--no-optimizations" {
				settings.optimize = false;
			} else if arg == "-c" || arg == "--compile" {
				settings.what_to_do = WhatToDo::Compile {
					target: CompileTarget::C,
					dst_file_path: None,
				};
			} else if let WhatToDo::Interpret { ref mut input } = settings.what_to_do {
				if arg == "-i" || arg == "--input" {
					*input = args.next();
				} else {
					panic!("unknown cmdline argument `{}` (for interpretation)", arg);
				}
			} else if let WhatToDo::Compile {
				ref mut dst_file_path,
				..
			} = settings.what_to_do
			{
				if arg == "-o" || arg == "--output-file" {
					*dst_file_path = args.next();
				} else {
					panic!("unknown cmdline argument `{}` (for compilation)", arg);
				}
			} else {
				unreachable!();
			}
		}
		settings
	}
}

#[derive(Debug)]
enum Prog {
	Raw(Vec<astraw::RawInstr>),
	Soup(Vec<astsoup::SoupInstr>),
}

fn main() {
	let settings = Settings::from_cmdline_args();
	if settings.verbose {
		dbg!(&settings);
	}
	if settings.help {
		println!("Help comming soon.");
	}

	let src_code = match settings.src {
		SrcSettings::Src(src_code) => src_code,
		SrcSettings::FilePath(src_file_path) => std::fs::read_to_string(src_file_path).expect("h"),
		SrcSettings::None => {
			println!("No source code, nothing to do.");
			return;
		}
	};
	if settings.verbose {
		dbg!(&src_code);
	}

	let parsing_result = parser::parse_instr_seq(&src_code);
	let mut prog = Prog::Raw(match parsing_result {
		Ok(prog) => prog,
		Err(error_vec) => {
			for error in error_vec {
				error.print(&src_code, None, true);
			}
			return;
		}
	});
	if settings.verbose {
		dbg!(&prog);
	}

	if settings.optimize {
		prog = Prog::Soup(astsoup::soupify(match prog {
			Prog::Raw(ref raw_prog) => raw_prog,
			_ => panic!("xxbf bug"),
		}));
		if settings.verbose {
			dbg!(&prog);
		}
	}

	match settings.what_to_do {
		WhatToDo::Interpret { input } => {
			let interact_with_user = input.is_some();
			let input = input.map(|s| s.bytes().collect());
			let output = match prog {
				Prog::Raw(raw_prog) => vm::run_raw(raw_prog, input),
				Prog::Soup(soup_prog) => vm::run_soup(soup_prog, input),
			};
			let output_string: String = output.iter().map(|&x| x as char).collect();
			if interact_with_user {
				println!("{}", output_string);
			}
		}
		WhatToDo::Compile {
			target,
			dst_file_path,
		} => {
			let output_code = match target {
				CompileTarget::C => match prog {
					Prog::Raw(raw_prog) => ctranspiler::transpile_raw_to_c(raw_prog),
					Prog::Soup(soup_prog) => ctranspiler::transpile_soup_to_c(soup_prog),
				},
			};
			if let Some(dst_file_path) = dst_file_path {
				std::fs::write(dst_file_path, output_code).expect("h");
			} else {
				print!("{}", output_code);
			}
		}
	}
}
