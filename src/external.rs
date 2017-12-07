use std::io::{self, Write};
use futures::{Future, Stream};
use hyper::Client;
use tokio_core::reactor::Core;
use service::Result;
use hyper_tls::HttpsConnector;

pub fn test() -> Result<()> {
    let mut core = Core::new()?;
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &handle).unwrap())
        .build(&handle);
    let uri = "https://www.github.com".parse()?;
    let work = client.get(uri).and_then(|res| {
        println!("Response: {}", res.status());

        res.body().for_each(|chunk| {
            io::stdout().write_all(&chunk).map_err(From::from)
        })
    });
    core.run(work)?;
    Ok(())
}

pub fn refund(order_id: &str, transaction_id: &str, refund_fee: i32) -> Result<()> {
    println!("{}{}{}", order_id, transaction_id, refund_fee);
    Ok(())
}