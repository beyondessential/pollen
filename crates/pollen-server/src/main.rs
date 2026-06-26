#[derive(Debug, clap::Parser)]
struct Args {
	#[command(flatten)]
	logging: lloggs::LoggingArgs,

	#[arg(long, short, default_value = "8080", env = "PORT")]
	port: u16,

	#[arg(long, env = "BIND_ADDRESS", conflicts_with = "port")]
	bind: Option<std::net::SocketAddr>,
}

#[tokio::main]
async fn main() -> miette::Result<()> {
	use std::net::{Ipv6Addr, SocketAddr, SocketAddrV6};

	use clap::Parser;
	use lloggs::PreArgs;
	use pollen_server::{server, state::AppState};

	let mut _guard = PreArgs::parse_with_env("POLLEN_LOG").setup()?;
	let args = Args::parse();
	if _guard.is_none() {
		_guard = Some(args.logging.setup(|v| match v {
			0 => "info",
			1 => "debug",
			_ => "trace",
		})?);
	}

	let addr = args
		.bind
		.unwrap_or_else(|| SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::LOCALHOST, args.port, 0, 0)));

	tokio::select! {
		_ = tokio::signal::ctrl_c() => {
			println!();
			tracing::info!("Received Ctrl+C signal, exiting");
		}
		res = server::serve(
			server::router(pollen_server::routes(AppState::init().await?)?),
			addr,
		) => {
			tracing::info!("Server exited");
			res?;
		}
	}
	Ok(())
}
