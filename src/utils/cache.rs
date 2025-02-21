use alloy_provider::{Provider, RootProvider};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

lazy_static::lazy_static! {
    static ref PRICE_CACHE: Mutex<HashMap<String, (u128, Instant)>> = Mutex::new(HashMap::new());
}

pub async fn cached_gas_price(provider: Arc<RootProvider>, ttl: Duration) -> eyre::Result<u128> {
    // Si ttl vaut 0, on n'utilise pas le cache
    if ttl == Duration::from_secs(0) {
        tracing::debug!("TTL is 0: bypassing cache");
        return Ok(provider.get_gas_price().await?);
    }

    const CACHE_KEY: &str = "gas_price";
    let mut cache = PRICE_CACHE.lock().await;

    if let Some((price, timestamp)) = cache.get(CACHE_KEY) {
        if timestamp.elapsed() < ttl {
            tracing::debug!("Gas price cache hit");
            return Ok(*price);
        }
        tracing::debug!("Gas price cache expired");
    }

    tracing::debug!("Fetching fresh gas price from provider");
    let gas_price = provider.get_gas_price().await?;

    // Mise à jour du cache
    cache.insert(CACHE_KEY.to_string(), (gas_price, Instant::now()));

    Ok(gas_price)
}

pub struct CachedGasPriceFuture {
    provider: Arc<RootProvider>,
    ttl: Duration,
    state: CacheState,
}

enum CacheState {
    Init,
    CheckingCache {
        cache_future: Pin<Box<dyn Future<Output = Option<(u128, Instant)>> + Send>>,
    },
    FetchingFromProvider {
        provider_future: Pin<Box<dyn Future<Output = eyre::Result<u128>> + Send>>,
    },
    UpdatingCache {
        gas_price: u128,
        update_future: Pin<Box<dyn Future<Output = ()> + Send>>,
    },
}

impl CachedGasPriceFuture {
    pub fn new(provider: Arc<RootProvider>, ttl: Duration) -> Self {
        Self {
            provider,
            ttl,
            state: CacheState::Init,
        }
    }
}

impl Future for CachedGasPriceFuture {
    type Output = eyre::Result<u128>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();

        loop {
            match &mut this.state {
                CacheState::Init => {
                    // Start by checking the cache
                    let cache_future = Box::pin(async {
                        let cache = PRICE_CACHE.lock().await;
                        cache.get("gas_price").map(|(price, time)| (*price, *time))
                    });

                    this.state = CacheState::CheckingCache { cache_future };
                }

                CacheState::CheckingCache { cache_future } => {
                    match Pin::new(cache_future).poll(cx) {
                        Poll::Ready(cache_result) => {
                            // Check if we got a valid cached value
                            if let Some((price, timestamp)) = cache_result {
                                if timestamp.elapsed() < this.ttl {
                                    tracing::debug!("Gas price cache hit (future)");
                                    return Poll::Ready(Ok(price));
                                }
                                tracing::debug!("Gas price cache expired (future)");
                            }

                            // Cache miss or expired, need to fetch from provider
                            let provider_clone = this.provider.clone();
                            let provider_future = Box::pin(async move {
                                provider_clone.get_gas_price().await.map_err(Into::into)
                            });

                            this.state = CacheState::FetchingFromProvider { provider_future };
                        }
                        Poll::Pending => return Poll::Pending,
                    }
                }

                CacheState::FetchingFromProvider { provider_future } => {
                    match Pin::new(provider_future).poll(cx) {
                        Poll::Ready(result) => {
                            match result {
                                Ok(gas_price) => {
                                    // Got price, now update cache
                                    let update_future = Box::pin(async move {
                                        let mut cache = PRICE_CACHE.lock().await;
                                        cache.insert(
                                            "gas_price".to_string(),
                                            (gas_price, Instant::now()),
                                        );
                                    });

                                    this.state = CacheState::UpdatingCache {
                                        gas_price,
                                        update_future,
                                    };
                                }
                                Err(e) => {
                                    return Poll::Ready(Err(e));
                                }
                            }
                        }
                        Poll::Pending => return Poll::Pending,
                    }
                }

                CacheState::UpdatingCache {
                    gas_price,
                    update_future,
                } => {
                    match Pin::new(update_future).poll(cx) {
                        Poll::Ready(_) => {
                            // Cache updated, return the gas price
                            return Poll::Ready(Ok(*gas_price));
                        }
                        Poll::Pending => return Poll::Pending,
                    }
                }
            }
        }
    }
}

/// Additional utility: cached block-based metrics
pub struct BlockMetricsCache {
    provider: Arc<RootProvider>,
    ttl: Duration,
    last_block: Mutex<Option<(u64, Instant)>>,
}

impl BlockMetricsCache {
    pub fn new(provider: Arc<RootProvider>, ttl: Duration) -> Self {
        Self {
            provider,
            ttl,
            last_block: Mutex::new(None),
        }
    }

    /// Get latest block number with caching
    pub async fn get_latest_block_number(&self) -> eyre::Result<u64> {
        let mut cache = self.last_block.lock().await;

        if let Some((block_number, timestamp)) = *cache {
            if timestamp.elapsed() < self.ttl {
                return Ok(block_number);
            }
        }

        let block_number = self.provider.get_block_number().await?;

        *cache = Some((block_number, Instant::now()));

        Ok(block_number)
    }
}
