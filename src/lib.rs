pub mod error;
pub mod sources;
use async_std::task;
use error::Result;
use futures::future::BoxFuture;
use sources::{
    alienvault, anubisdb, binaryedge, bufferover, c99, certspotter, crtsh, facebook, hackertarget,
    intelx, passivetotal, spyse, sublister, threatcrowd, threatminer, urlscan, virustotal, wayback,
};
use std::collections::HashSet;
use std::sync::Arc;

// wrapper result type

trait IntoSubdomain {
    fn subdomains(&self) -> HashSet<String>;
}

// Collects data from all sources which don't require and API key
async fn free_sources(host: Arc<String>) -> HashSet<String> {
    let mut tasks = Vec::new();
    let mut results = HashSet::new();
    let sources: Vec<BoxFuture<Result<HashSet<String>>>> = vec![
        Box::pin(anubisdb::run(Arc::clone(&host))),
        Box::pin(alienvault::run(Arc::clone(&host))),
        Box::pin(bufferover::run(Arc::clone(&host), true)),
        Box::pin(bufferover::run(Arc::clone(&host), false)),
        Box::pin(certspotter::run(Arc::clone(&host))),
        Box::pin(crtsh::run(Arc::clone(&host))),
        Box::pin(threatcrowd::run(Arc::clone(&host))),
        Box::pin(urlscan::run(Arc::clone(&host))),
        Box::pin(virustotal::run(Arc::clone(&host))),
        Box::pin(threatminer::run(Arc::clone(&host))),
        Box::pin(sublister::run(Arc::clone(&host))),
        Box::pin(wayback::run(Arc::clone(&host))),
        Box::pin(hackertarget::run(host)),
    ];

    for s in sources {
        tasks.push(task::spawn(async { s.await }));
    }

    for t in tasks {
        t.await
            .iter()
            .flatten()
            .map(|s| results.insert(s.into()))
            .for_each(drop);
    }

    results
}

// Collects data from all sources
async fn all_sources(host: Arc<String>) -> HashSet<String> {
    let mut tasks = Vec::new();
    let mut results = HashSet::new();
    let sources: Vec<BoxFuture<Result<HashSet<String>>>> = vec![
        Box::pin(anubisdb::run(Arc::clone(&host))),
        Box::pin(binaryedge::run(Arc::clone(&host))),
        Box::pin(alienvault::run(Arc::clone(&host))),
        Box::pin(bufferover::run(Arc::clone(&host), true)),
        Box::pin(bufferover::run(Arc::clone(&host), false)),
        Box::pin(certspotter::run(Arc::clone(&host))),
        Box::pin(crtsh::run(Arc::clone(&host))),
        Box::pin(threatcrowd::run(Arc::clone(&host))),
        Box::pin(urlscan::run(Arc::clone(&host))),
        Box::pin(virustotal::run(Arc::clone(&host))),
        Box::pin(threatminer::run(Arc::clone(&host))),
        Box::pin(sublister::run(Arc::clone(&host))),
        Box::pin(wayback::run(Arc::clone(&host))),
        Box::pin(facebook::run(Arc::clone(&host))),
        Box::pin(spyse::run(Arc::clone(&host))),
        Box::pin(c99::run(Arc::clone(&host))),
        Box::pin(intelx::run(Arc::clone(&host))),
        Box::pin(passivetotal::run(Arc::clone(&host))),
        Box::pin(hackertarget::run(host)),
    ];

    for s in sources {
        tasks.push(task::spawn(async { s.await }));
    }

    for t in tasks {
        t.await
            .iter()
            .flatten()
            .map(|s| results.insert(s.into()))
            .for_each(drop);
    }

    results
}

// Takes a bunch of hosts and collects data on them
pub async fn runner(hosts: Vec<String>, all: bool) -> Vec<String> {
    // the number of root domains to fetch data on at a one time
    const ACTIVE_REQUESTS: usize = 200;
    use futures::stream::StreamExt;

    let responses = futures::stream::iter(hosts.into_iter().map(|host| {
        let host = Arc::new(host);
        task::spawn(async move {
            if all {
                all_sources(host).await
            } else {
                free_sources(host).await
            }
        })
    }))
    .buffer_unordered(ACTIVE_REQUESTS)
    .collect::<Vec<HashSet<String>>>();

    responses.await.into_iter().flatten().collect()
}
