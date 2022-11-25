linux:
	rustc src/main.rs -o ffk --target x86_64-unknown-linux-gnu

windows:
	rustc src/main.rs -o ffk.exe --target x86_64-pc-windows-gnu