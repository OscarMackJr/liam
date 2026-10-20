pub mod hometown;
pub mod olive;

use reqwest::{
    blocking::Client,
    dns::{Name, Resolve, Resolving},
    Proxy,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Default)]
pub struct DenyAllResolver {
    calls: AtomicUsize,
}

impl DenyAllResolver {
    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::Relaxed)
    }
}

impl Resolve for DenyAllResolver {
    fn resolve(&self, _name: Name) -> Resolving {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Box::pin(async { Err("liam DNS resolution is forbidden".into()) })
    }
}

pub struct ProxyClient {
    client: Client,
    resolver: Arc<DenyAllResolver>,
}

impl ProxyClient {
    pub fn client(&self) -> Client {
        self.client.clone()
    }

    pub fn resolver_calls(&self) -> usize {
        self.resolver.calls()
    }
}

pub fn build_proxy_client(proxy: &str) -> Result<ProxyClient, reqwest::Error> {
    let resolver = Arc::new(DenyAllResolver::default());
    let client = Client::builder()
        .no_proxy()
        .proxy(Proxy::all(proxy)?)
        .dns_resolver(Arc::clone(&resolver))
        .build()?;
    Ok(ProxyClient { client, resolver })
}

#[cfg(test)]
mod tests {
    use super::{build_proxy_client, DenyAllResolver};
    use reqwest::dns::Resolve;
    use std::{
        future::Future,
        pin::Pin,
        task::{Context, Poll, Waker},
    };

    #[test]
    fn deny_all_resolver_starts_with_no_calls() {
        assert_eq!(DenyAllResolver::default().calls(), 0);
    }

    #[test]
    fn loopback_proxy_client_is_built_with_a_resolver_that_has_not_run() {
        let client = build_proxy_client("http://127.0.0.1:50413").expect("build client");
        assert_eq!(client.resolver_calls(), 0);
    }

    #[test]
    fn deny_all_resolver_fails_closed_on_every_lookup() {
        let resolver = DenyAllResolver::default();
        let mut resolution = resolver.resolve("hometown.test".parse().expect("valid name"));
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Pin::new(&mut resolution).poll(&mut context),
            Poll::Ready(Err(_))
        ));
        assert_eq!(resolver.calls(), 1);
    }
}
