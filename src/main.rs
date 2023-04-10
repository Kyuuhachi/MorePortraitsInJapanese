#![feature(decl_macro, let_chains, assert_matches)]

use std::path::PathBuf;
use std::fs;

use clap::Parser;
use regex::Regex;

use themelios::scena::ed7::{Scena, read as read_scena, write as write_scena};
use themelios::scena::code::{Insn, FlatInsn};
use themelios::text::TextSegment;
use themelios::types::TString;


#[derive(Debug, Clone, clap::Parser)]
struct Cli {
	which: CliGame,
	#[clap(long, short='P', value_hint = clap::ValueHint::DirPath)]
	portraits: PathBuf,
	#[clap(long, short, value_hint = clap::ValueHint::DirPath)]
	japanese: PathBuf,
	#[clap(long, short, value_hint = clap::ValueHint::DirPath)]
	out: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum CliGame {
	Zero,
	Azure,
}

fn main() -> anyhow::Result<()> {
	let cli = Cli::parse();

	let game = match cli.which {
		CliGame::Zero => themelios::types::Game::ZeroKai,
		CliGame::Azure => themelios::types::Game::AoKai,
	};

	if cli.out.exists() {
		fs::remove_dir_all(&cli.out)?;
	}

	std::fs::create_dir_all(cli.out.join("scena"))?;

	let mut i = std::fs::read_dir(&cli.portraits.join("data/scena_us"))?.collect::<Result<Vec<_>, _>>()?;
	i.sort_by_key(|a| a.path());
	for file in i {
		let p = file.path();
		let name = p.file_name().unwrap().to_str().unwrap().split_once('.').unwrap().0;
		println!("{name}");
		let p2 = cli.japanese.join("data/scena").join(p.file_name().unwrap());
		let p3 = cli.out.join("scena").join(p.file_name().unwrap());

		let scena_p = read_scena(game, &std::fs::read(&p)?)?;
		let scena_jp = read_scena(game, &std::fs::read(&p2)?)?;
		let scena = merge(cli.which, name, &scena_p, &scena_jp);
		std::fs::write(&p3, write_scena(game, &scena)?)?;
	}

	Ok(())
}

fn merge(game: CliGame, name: &str, p: &Scena, jp: &Scena) -> Scena {
	let mut p = p.clone();
	assert_eq!(p.functions.len(), jp.functions.len());
	assert_eq!(p.npcs.len(), jp.npcs.len());
	assert_eq!(p.labels.iter().flatten().count(), jp.labels.iter().flatten().count());
	p.npcs.iter_mut().zip(jp.npcs.iter()).for_each(|a| *a.0 = a.1.clone());
	p.labels.iter_mut().flatten().zip(jp.labels.iter().flatten()).for_each(|a| *a.0 = a.1.clone());

	for (i, (p, jp)) in p.functions.iter_mut().zip(jp.functions.iter()).enumerate() {
		#[derive(Debug, Clone, PartialEq)]
		enum T {
			Text(Vec<TextSegment>),
			TString(TString),
		}
		let mut t = Vec::new();

		// Collect the strings from the japanese version
		for insn in &jp.0 {
			if let FlatInsn::Insn(insn) = insn {
				macro run {
					([$(($ident:ident $(($_n:ident $($ty:tt)*))*))*]) => {
						match insn {
							$(Insn::$ident($($_n),*) => {
								$(run!($_n $($ty)*);)*
							})*
						}
					},
					($v:ident Text) => {
						for f in &$v.0 {
							t.push(T::Text(f.clone()));
						}
					},
					($v:ident TString) => {
						t.push(T::TString($v.clone()));
					},
					($v:ident Vec<TString>) => {
						for f in $v.iter() {
							t.push(T::TString(f.clone()));
						}
					},
					($i:ident $($t:tt)*) => {}
				}
				themelios::scena::code::introspect!(run);
			}
		}

		let ts = |s: &str| T::TString(TString(String::from(s)));
		// Misc fixup
		match game {
			CliGame::Zero => match (name, i) {
				("c0140",  8) => t.insert(0, ts("ウェンディ")),
				("c0140", 16) => t.insert(0, ts("ウェンディ")),
				("c0140", 17) => t.insert(0, ts("ウェンディ")),
				("c0210",  7) => t.insert(0, ts("オスカー")),
				("c0210", 26) => t.insert(1, ts("オスカー")),
				("c0210", 29) => t.insert(0, ts("オスカー")),
				("c0210", 31) => t.insert(0, ts("ベネット")),
				("c020c", 34) => {
					assert_eq!(t.remove(10), ts("少年"));
					assert_eq!(t.remove(31), ts("少年"));
				},
				("c0240", 25) => {
					assert_eq!(t.remove(17), ts("少年"));
					assert_eq!(t.remove(37), ts("少年"));
					assert_eq!(t.remove(38), ts("少年"));
				}
				("c0240", 29) => {
					assert_eq!(t.remove(12), ts("少年"));
					assert_eq!(t.remove(16), ts("少年"));
					assert_eq!(t.remove(23), ts("少年"));
					assert_eq!(t.remove(24), ts("少年"));
					assert_eq!(t.remove(25), ts("少年"));
					assert_eq!(t.remove(30), ts("少年"));
					assert_eq!(t.remove(31), ts("少年"));
					assert_eq!(t.remove(32), ts("少年"));
					assert_eq!(t.remove(33), ts("少年"));
				}
				("c1010", 19) => {
					t.insert(0, ts("遊撃士スコット"));
					t.insert(3, ts("遊撃士ヴェンツェル"));
				}
				("c1010", 20) => {
					t.insert(0, ts("遊撃士リン"));
					t.insert(2, ts("遊撃士エオリア"));
				}
				("c1010", 21) => t.insert(10, ts("受付ミシェル")),
				("c1010", 22) => t.insert(2, ts("受付ミシェル")),
				("c1010", 31) => t.insert(3, ts("受付ミシェル")),
				("c1150", 29) => t.insert(0, ts("ピエール副局長")),
				("c1160",  2) => t.insert(0, ts("ピエール副局長")),
				("c1160",  4) => t.insert(2, ts("ピエール副局長")),
				("c1400", 26) => assert_eq!(t.remove(41), ts("禿の大男")),
				("c1410", 51) => assert_eq!(t.remove(95), ts("禿の大男")),
				("c1410", 55) => t.insert(0, ts("アッバス")),
				("t2020", 10) => t.insert(0, ts("ミレイユ准尉")),
				_ => {}
			},
			CliGame::Azure => match (name, i) {
				("c0140", 25) => t.insert(0, ts("ウェンディ")),
				("c0210", 28) => {
					t.insert(13, t[10].clone());
					t.insert(14, t[11].clone());
				},
				("c1010", 21) => {
					t.insert(0, ts("遊撃士リン"));
					t.insert(2, ts("遊撃士エオリア"));
				}
				("c1010", 22) => {
					t.insert(0, ts("遊撃士スコット"));
					t.insert(2, ts("遊撃士ヴェンツェル"));
				}
				("m1060", 4) => assert_eq!(t.remove(5), ts("騎士装束の娘")),
				("m1140", 7) => assert_eq!(t.remove(5), ts("騎士装束の娘")),
				("t1310", 57) => {
					std::assert_matches::assert_matches!(
						p.0.remove(238),
						FlatInsn::Insn(Insn::TextTalk(themelios::types::CharId(14), _)),
					);
				},
				("t1650", 4) => assert_eq!(t.remove(5), ts("女医")),
				("t2520", 20) => {
					// interim until Shin fixes it
					let T::Text(b) = t.remove(8) else { panic!() };
					let T::Text(a) = &mut t[7] else { panic!() };
					a.extend(b);
				}
				("t2520", 22) => {
					t.insert(23, t[19].clone());
					t.insert(24, t[20].clone());
					t.insert(25, t[21].clone());
				}

				// Chests. They're supposed to have different text.
				("m4210", 5..=9) => continue,
				("m4220", 5..=10) => continue,
				("r1010", 7..=10) => continue,
				("r1580", 7..=8) => continue,
				_ => {}
			}
		}

		// Insert japanese text into portraits.
		let mut j = 0;
		for insn in &mut p.0 {
			if let FlatInsn::Insn(insn) = insn {
				macro run {
					([$(($ident:ident $(($_n:ident $($ty:tt)*))*))*]) => {
						match insn {
							$(Insn::$ident($($_n),*) => {
								$(run!($_n $($ty)*);)*
							})*
						}
					},
					($v:ident Text) => {
						for f in &mut $v.0 {
							let Some(T::Text(g)) = t.get(j) else { panic!("[{name}:{i}:{j}] expected Text: {:?} ⇒ {:?}", f, t.get(j)); };
							j += 1;
							insert_portrait(f, g);
						}
					},
					($v:ident TString) => {
						let Some(T::TString(g)) = t.get(j) else { panic!("[{name}:{i}:{j}] expected TString: {:?} ⇒ {:?}", $v, t.get(j)); };
						j += 1;
						*$v = g.clone();
					},
					($v:ident Vec<TString>) => {
						for f in $v.iter_mut() {
							let Some(T::TString(g)) = t.get(j) else { panic!("[{name}:{i}:{j}] expected TString: {:?} ⇒ {:?}", $v, t.get(j)); };
							j += 1;
							*f = g.clone();
						}
					},
					($i:ident $($t:tt)*) => {}
				}
				themelios::scena::code::introspect!(run);
			}
		}

		if j != t.len() {
			panic!("[{name}:{i}:{j}] mismatch: {j} ≠ {t:?}.len()");
		}
	}
	p
}

fn insert_portrait(a: &mut Vec<TextSegment>, b: &[TextSegment]) {
	lazy_static::lazy_static! {
		static ref FACE: Regex = Regex::new(r"#\d*F").unwrap();
		static ref POS: Regex = Regex::new(r"#\d*P").unwrap();
		static ref VOICE: Regex = Regex::new(r"#\d*V").unwrap();
	}
	let port = text2str(a);
	let mut jp = text2str(b);
	let pos = VOICE.find(&jp).map_or(0, |f| f.end());

	match (POS.find(&jp), POS.find(&port)) {
		(Some(a), Some(b)) => {
			let (s,e) = (a.start(), a.end());
			jp.drain(s..e);
			jp.insert_str(s, b.as_str())
		}
		(None, Some(b)) => {
			jp.insert_str(pos, b.as_str())
		}
		(Some(a), None) => {
			jp.drain(a.start()..a.end());
		}
		(None, None) => { }
	}

	if FACE.find(&jp).is_none() && let Some(face) = FACE.find(&port) {
		jp.insert_str(pos, face.as_str())
	}

	if jp != text2str(b) {
		println!("{:?}", text2str(b));
		println!("{:?}", port);
		println!("{:?}", jp);
		println!();
	}

	*a = str2text(&jp);
}

fn text2str(t: &[TextSegment]) -> String {
	let mut s = String::new();
	for i in t {
		match i {
			TextSegment::String(v) => s.push_str(v),
			TextSegment::Line => s.push('\n'),
			TextSegment::Wait => s.push_str("{wait}"),
			TextSegment::Color(v) => s.push_str(&format!("{{color {v}}}")),
			TextSegment::Item(v) => s.push_str(&format!("{{item {v}}}", v=v.0)),
			TextSegment::Byte(v) => s.push_str(&format!("{{#{v:02X}}}")),
		}
	}
	s
}

fn str2text(s: &str) -> Vec<TextSegment> {
	lazy_static::lazy_static! {
		static ref SEGMENT: Regex = Regex::new(r"(?x)
			(?P<t>.*?)
			(?:
				(?P<line>)\n|
				(?P<line2>)\r|
				(?P<wait>)\{wait\}|
				(?P<page>)\{page\}|
				\{color\ (?P<color>\d+)\}|
				\{item\ (?P<item>\d+)\}|
				\{\#(?P<hex>[[:xdigit:]]{2})\}|
				$
			)
		").unwrap();
	}
	let mut out = Vec::new();
	for c in SEGMENT.captures_iter(s) {
		if let Some(t) = c.name("t") {
			if !t.as_str().is_empty() {
				out.push(TextSegment::String(t.as_str().to_owned()))
			}
		}
		if c.name("line").is_some() {
			out.push(TextSegment::Line)
		}
		if c.name("wait").is_some() {
			out.push(TextSegment::Wait)
		}
		if c.name("page").is_some() {
			out.push(TextSegment::Byte(0x03))
		}
		if let Some(c) = c.name("color") {
			out.push(TextSegment::Color(c.as_str().parse().unwrap()))
		}
		if let Some(c) = c.name("item") {
			out.push(TextSegment::Item(c.as_str().parse::<u16>().unwrap().into()))
		}
		if let Some(c) = c.name("hex") {
			out.push(TextSegment::Byte(u8::from_str_radix(c.as_str(), 16).unwrap()))
		}
	}
	out
}

