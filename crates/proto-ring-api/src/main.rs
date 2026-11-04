#![forbid(unsafe_code)]

fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let code = proto_ring_api::run_cli(
        &args,
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock(),
    );
    std::process::ExitCode::from(code as u8)
}
