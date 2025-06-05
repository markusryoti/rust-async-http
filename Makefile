build:
	cargo build --release

load:
	wrk -t4 -c100 -d10s http://127.0.0.1:7878
