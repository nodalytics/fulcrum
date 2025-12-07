use std::{str::FromStr, sync::Arc, time::Duration};

mod cli;
use cli::*;

use ethers_providers::{Middleware, Provider};
use ethers_signers::{LocalWallet, Signer};

use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_subscriber::{fmt, EnvFilter};
use tracing_appender::{rolling, non_blocking};
use tracing_appender::non_blocking::{WorkerGuard};

use fulcrum_engine::{
    constant::arbitrum::{UNISWAP_V3_FACTORY, UNISWAP_V3_INIT_CODE_HASH},
    prices_at,
    types::{Address, ExchangeId, Pair, Position, Token},
    uniswap_v3::{self},
    Engine, FulcrumExecutor, OrderService, PriceGraph, PriceService,
};
use fulcrum_sequencer_feed::SequencerFeed;
use fulcrum_ws_cli::FastWsClient;

use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() {
    // init logger crate
    let _guard = init_tracing();
    tracing::info!(
        r#"
        █▀▀ █░█ █░░ █▀▀ █▀█ █░█ █▀▄▀█
        █▀░ █▄█ █▄▄ █▄▄ █▀▄ █▄█ █░▀░█
        "#
    );
    tracing::info!("🚀 Starting...");
    // pin to core
    // tuna --cpus 1-7 --isolate, 0 becomes core 1s
    let core_ids = core_affinity::get_core_ids().unwrap();
    core_affinity::set_for_current(core_ids[0]);

    // Load cli args
    let FulcrumCli {
        ws,
        chain,
        sub_command,
    } = argh::from_env();

    let ws_endpoint = ws;
    let provider = Provider::new(
        FastWsClient::connect(ws_endpoint)
            .await
            .expect("provider connects"),
    );

    let (uniswap_v2_pairs, uniswap_v3_pairs) = load_pairs();

    // Price fetch
    if let SubCommand::Prices(PricesCommand { at }) = sub_command {
        info!("querying prices at block: #{at}, chain: {:?}", chain);
        let price_service = PriceService::new(
            Arc::new(provider),
            uniswap_v2_pairs.as_slice(),
            uniswap_v3_pairs.as_slice(),
        );
        prices_at(price_service, at).await;
        // TODO: graceful shutdown
        return;
    }

    // Run engine
    if let SubCommand::Run(RunCommand {
        key,
        min_profit,
        executor,
        dry_run,
    }) = sub_command
    {
        let wallet = key
            .expect("--key given")
            .parse::<LocalWallet>()
            .expect("valid secret key")
            .with_chain_id(chain);

        let provider = Arc::new(
            provider
                .with_sender(wallet.address())
                .set_interval(Duration::from_millis(100))
                .clone(),
        );

        let executor_contract = FulcrumExecutor::new(executor, Arc::clone(&provider));
        let order_service = OrderService::new(
            Arc::clone(&provider),
            chain,
            executor_contract,
            wallet.clone(),
        )
        .await;
        let sequencer_feed = SequencerFeed::arbitrum_one().await;
        let price_service = PriceService::new(
            Arc::clone(&provider),
            uniswap_v2_pairs.as_slice(),
            uniswap_v3_pairs.as_slice(),
        );

        info!(
            "monitoring chain: {:?}\nsigning with: {:?}\nexecutor: {:?}\npassive: {dry_run}",
            chain,
            wallet.address(),
            executor,
        );
        let ws_latency = provider.provider().as_ref().report_latency().await;
        info!("~ws latency: ~{:?}ms", ws_latency);
        info!(
            "min. profit margin: {:?}%\npairs: {:#?}{:#?}\n",
            min_profit, uniswap_v3_pairs, uniswap_v2_pairs,
        );

        // build trade search paths
        let pairs: Vec<Pair> = uniswap_v3_pairs.iter().map(|(p, _)| *p).collect(); // TODO: include v2 pairs
        let weth_paths = PriceGraph::find_paths(Token::WETH, pairs.as_slice());
        let arb_paths = PriceGraph::find_paths(Token::ARB, pairs.as_slice());
        let magic_paths = PriceGraph::find_paths(Token::MAGIC, pairs.as_slice());
        let usdt_paths = PriceGraph::find_paths(Token::USDT, pairs.as_slice());
        let usdc_paths = PriceGraph::find_paths(Token::USDC, pairs.as_slice());
        let usdce_paths = PriceGraph::find_paths(Token::USDCe, pairs.as_slice());
        // via flash loans position can be anything
        // positions should be big enough to make profits, small enough to not cross v3 liquidity ticks
        let all_paths = [
            (Position::of(5_000, Token::USDC), usdc_paths.as_ref()),
            (Position::of(5_000, Token::USDCe), usdce_paths.as_ref()),
            (Position::of(3, Token::WETH), weth_paths.as_ref()),
            (Position::of(5_000, Token::USDT), usdt_paths.as_ref()),
            (Position::of(4_500, Token::ARB), arb_paths.as_ref()),
            (Position::of(10_000, Token::MAGIC), magic_paths.as_ref()),
        ];

        let engine = Engine::new(price_service, order_service, sequencer_feed);
        engine.run(&all_paths, min_profit, dry_run).await;
    }
}

/// Load the active trading pairs (uniswapv2, uniswapv3)
fn load_pairs() -> (Vec<(Pair, Address)>, Vec<(Pair, Address)>) {
    // only these v3 pairs have sufficient liquidity
    let pairs: &[Pair] = &[
        Pair::new(Token::USDC, Token::WETH, 100, ExchangeId::Uniswap),
        Pair::new(Token::USDC, Token::WETH, 500, ExchangeId::Uniswap),
        Pair::new(Token::USDC, Token::WETH, 3_000, ExchangeId::Uniswap),
        Pair::new(Token::USDCe, Token::WETH, 100, ExchangeId::Uniswap),
        Pair::new(Token::USDCe, Token::WETH, 500, ExchangeId::Uniswap),
        Pair::new(Token::USDCe, Token::WETH, 3_000, ExchangeId::Uniswap),
        Pair::new(Token::USDC, Token::ARB, 500, ExchangeId::Uniswap),
        Pair::new(Token::USDCe, Token::ARB, 500, ExchangeId::Uniswap),
        Pair::new(Token::WETH, Token::ARB, 100, ExchangeId::Uniswap),
        Pair::new(Token::WETH, Token::ARB, 500, ExchangeId::Uniswap),
        Pair::new(Token::WETH, Token::ARB, 3_000, ExchangeId::Uniswap),
        Pair::new(Token::WETH, Token::USDT, 500, ExchangeId::Uniswap),
        Pair::new(Token::WETH, Token::USDT, 100, ExchangeId::Uniswap),
        Pair::new(Token::USDT, Token::USDC, 100, ExchangeId::Uniswap),
        Pair::new(Token::USDT, Token::USDCe, 100, ExchangeId::Uniswap),
        Pair::new(Token::MAGIC, Token::WETH, 500, ExchangeId::Uniswap),
        Pair::new(Token::MAGIC, Token::USDC, 500, ExchangeId::Uniswap),
        Pair::new(Token::MAGIC, Token::USDCe, 500, ExchangeId::Uniswap),
    ];
    let uniswap_v3_pairs: Vec<(Pair, Address)> = pairs
        .iter()
        .map(|p| {
            let pool_address = uniswap_v3::pool_address_from_pair(
                *p,
                UNISWAP_V3_FACTORY.into(),
                &UNISWAP_V3_INIT_CODE_HASH,
            );
            (*p, pool_address)
        })
        .collect();

    let chronos_pairs: &[(Pair, Address)] = &[
        (
            Pair::new(Token::WETH, Token::ARB, 200, ExchangeId::Chronos),
            Address::from_str("afe909b1a5ed90d36f9ee1490fcb855645c00eb3").unwrap(),
        ),
        (
            Pair::new(Token::WETH, Token::USDCe, 200, ExchangeId::Chronos),
            Address::from_str("A2F1C1B52E1b7223825552343297Dc68a29ABecC").unwrap(),
        ),
        (
            Pair::new(Token::WETH, Token::USDT, 200, ExchangeId::Chronos),
            Address::from_str("8a263cc1dfdce6c64e2a1cf6133c22eed5d4e29d").unwrap(),
        ),
    ];
    let sushi_pairs: &[(Pair, Address)] = &[(
        Pair::new(Token::WETH, Token::USDCe, 300, ExchangeId::Sushi),
        Address::from_str("905dfcd5649217c42684f23958568e533c711aa3").unwrap(),
    )];
    let camelot_pairs: &[(Pair, Address)] = &[
        (
            Pair::new(Token::WETH, Token::ARB, 300, ExchangeId::Sushi),
            Address::from_str("a6c5c7d189fa4eb5af8ba34e63dcdd3a635d433f").unwrap(),
        ),
        (
            Pair::new(Token::WETH, Token::USDCe, 300, ExchangeId::Sushi),
            Address::from_str("84652bb2539513baf36e225c930fdd8eaa63ce27").unwrap(),
        ),
    ];
    let uniswap_v2_pairs = [chronos_pairs, sushi_pairs, camelot_pairs].concat();
    (uniswap_v2_pairs, uniswap_v3_pairs)
}


pub fn init_tracing() -> WorkerGuard {
    // File appender (daily rotation)
    let file_appender = rolling::daily("logs", "fulcrum.log");
    let (non_blocking, guard) = non_blocking(file_appender);

    // Try to build EnvFilter from environment (RUST_LOG)
    let filter = match EnvFilter::try_from_default_env() {
        Ok(f) => f,
        Err(e) => {
            error!(
                "⚠️ Failed to load RUST_LOG from environment: {}. Falling back to 'info,error'.",
                e
            );

            // Attempt fallback with explicit "info,error"
            // TODO: fix fallback
            match EnvFilter::try_new("info,warn,error") {
                Ok(fallback) => {
                    error!("ℹ️ Using fallback EnvFilter = 'info,warn,error'.");
                    fallback
                }
                Err(inner_err) => {
                    error!(
                        "❌ Unexpected error initializing fallback EnvFilter('info,warn,error'): {}. Using default.",
                        inner_err
                    );
                    EnvFilter::default()
                }
            }
        }
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stdout)) // console output
        .with(fmt::layer().with_writer(non_blocking))    // file output
        .with(filter)
        .init();

    info!("✅ Tracing initialized with info+warn+error levels.");
    guard
}