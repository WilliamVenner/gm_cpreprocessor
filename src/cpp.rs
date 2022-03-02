use std::{path::{Path, PathBuf}, process::{Command, Stdio}, io::Write, ffi::OsStr};

type Result<T> = std::result::Result<T, Error>;

lazy_static::lazy_static! {
	static ref COMPILER: Option<Compiler> = match Compiler::find() {
		Some(compiler) => Some(compiler),
		None => {
			eprintln!("Failed to find C preprocessor on this system!!!");
			None
		}
	};
}

pub enum Error {
	Error(Box<dyn std::error::Error>),
	String(String)
}
impl std::fmt::Debug for Error {
	#[inline]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Error(arg0) => std::fmt::Debug::fmt(arg0, f),
			Self::String(arg0) => std::fmt::Display::fmt(arg0, f),
		}
	}
}
impl std::fmt::Display for Error {
	#[inline]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Error(arg0) => std::fmt::Display::fmt(arg0, f),
			Self::String(arg0) => std::fmt::Display::fmt(arg0, f),
		}
	}
}
impl<E: std::error::Error + 'static> From<E> for Error {
	#[inline]
	fn from(value: E) -> Self {
		Self::Error(Box::new(value) as _)
	}
}

fn capture(process: std::process::Child) -> Result<Vec<u8>> {
	let output = process.wait_with_output()?;
	if !output.status.success() {
		return Err(Error::String(format!("Error code: {}\n\n{}", output.status.code().unwrap_or(-1), String::from_utf8_lossy(&output.stderr))));
	}

	Ok(output.stdout)
}

pub enum Compiler {
	GnuClang(PathBuf),
	Msvc(PathBuf)
}
impl Compiler {
	fn find() -> Option<Self> {
		std::env::set_var("OPT_LEVEL", "0");
		std::env::set_var("TARGET", env!("TARGET"));
		std::env::set_var("HOST", env!("HOST"));

		let tool = match cc::Build::new().cargo_metadata(false).try_get_compiler() {
			Ok(tool) => tool,
			Err(_) => return None
		};

		if tool.is_like_clang() || tool.is_like_gnu() {
			Some(Self::GnuClang(tool.path().to_path_buf()))
		} else if tool.is_like_msvc() {
			Some(Self::Msvc(tool.path().to_path_buf()))
		} else {
			None
		}
	}

	fn preprocess(&self, code: &[u8]) -> Result<Vec<u8>> {
		match self {
			Self::GnuClang(path) => Self::gnu_clang(path, code),
			Self::Msvc(path) => Self::msvc(path, code)
		}
	}

	// Arguments are the same for Clang and Gnu gcc
	fn gnu_clang(path: &Path, code: &[u8]) -> Result<Vec<u8>> {
		let mut process = Command::new(path)
			.args(&["-nostdinc", "-P", "-E", "-x", "c", "-"])
			.stdin(Stdio::piped())
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.spawn()?;

		process.stdin.as_mut().unwrap().write_all(code)?;

		capture(process)
	}

	fn msvc(path: &Path, code: &[u8]) -> Result<Vec<u8>> {
		#[repr(transparent)]
		struct TempFileHandle(PathBuf);
		impl Drop for TempFileHandle {
			fn drop(&mut self) {
				let _ = std::fs::remove_file(&self.0);
			}
		}
		impl AsRef<Path> for TempFileHandle {
			#[inline]
			fn as_ref(&self) -> &Path {
				&self.0
			}
		}
		impl AsRef<OsStr> for TempFileHandle {
			#[inline]
			fn as_ref(&self) -> &OsStr {
				self.0.as_os_str()
			}
		}

		let src_path = TempFileHandle(std::env::temp_dir().join(format!("{}.c", uuid::Uuid::new_v4())));
		std::fs::write(&src_path, code)?;

		capture({
			Command::new(path)
				.args(&["/EP", "/X"])
				.arg(&src_path)
				.stdin(Stdio::piped())
				.stdout(Stdio::piped())
				.stderr(Stdio::piped())
				.spawn()?
		})
	}
}

pub fn can_preprocess() -> bool {
	COMPILER.is_some()
}

pub fn preprocess(code: &[u8]) -> Option<Result<Vec<u8>>> {
	COMPILER.as_ref().map(|compiler| compiler.preprocess(code))
}
