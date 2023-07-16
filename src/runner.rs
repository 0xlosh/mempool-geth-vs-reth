use std::{collections::HashMap, time::{UNIX_EPOCH, SystemTime}};
use eyre::Result;
use tracing::{debug, info};
use console::style;
use ethers::prelude::*;
use crate::hwi::*;

pub async fn execute(geth: Provider<HWI>, reth: Provider<HWI>, count: usize) -> Result<()> {
    let handle = tokio::runtime::Handle::current();
    debug!("{:?}", handle.runtime_flavor());

    assert_eq!(
        geth.get_block_number().await?,
        reth.get_block_number().await?,
    );

    let to = "0x3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad".parse::<H160>()?; // uniswap router

    let t_geth = tokio::spawn(async move {
        let mut mapping: HashMap<H256, u128> = HashMap::new();

        loop {
            let mut stream = geth.subscribe_full_pending_txs().await.unwrap();

            while let Some(tx) = stream.next().await {
                if tx.to.is_none() || tx.to.unwrap() != to {
                    continue;
                }

                let timestamp_millis = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                mapping.insert(tx.hash, timestamp_millis);

                info!("geth: {:?} {}", tx.hash, timestamp_millis);

                if mapping.len() >= count {
                    break;
                }
            }

            if mapping.len() >= count {
                break;
            }
        }

        mapping
    });

    let t_reth = tokio::spawn(async move {
        let mut mapping: HashMap<H256, u128> = HashMap::new();

        loop {
            let mut stream = reth.subscribe_pending_txs().await.unwrap().transactions_unordered(usize::MAX);

            while let Some(Ok(tx)) = stream.next().await {
                if tx.to.is_none() || tx.to.unwrap() != to {
                    continue;
                }

                let timestamp_millis = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                mapping.insert(tx.hash, timestamp_millis);
                
                info!("reth: {:?} {}", tx.hash, timestamp_millis);

                if mapping.len() >= count {
                    break;
                }
            }

            if mapping.len() >= count {
                break;
            }
        }

        mapping
    });

    let (m_geth, m_reth) = match tokio::join!(t_geth, t_reth) {
        (Ok(m_geth), Ok(m_reth)) => (m_geth, m_reth),
        (Err(e), _) => panic!("geth error: {}", e),
        (_, Err(e)) => panic!("reth error: {}", e),
    };

    let keys_geth = m_geth.keys().copied().collect::<Vec<_>>();
    let keys_reth = m_reth.keys().copied().collect::<Vec<_>>();
    let keys = keys_geth.into_iter().filter(|k| keys_reth.contains(k)).collect::<Vec<H256>>();

    let mut c_geth = 0;
    let mut c_reth = 0;

    for k in keys {
        let t_geth = m_geth.get(&k).unwrap();
        let t_reth = m_reth.get(&k).unwrap();

        if t_geth < t_reth {
            c_geth += 1;
            println!("{}", style(format!("geth: {}, reth: {} | diff: {} ms", t_geth, t_reth, t_reth - t_geth)).yellow());
        } 

        if t_reth < t_geth {
            c_reth += 1;
            println!("{}",  style(format!("geth: {}, reth: {} | diff: {} ms", t_geth, t_reth, t_geth - t_reth)).red()); 
        }
    }

    println!("geth: {}, reth: {}", c_geth, c_reth);

    Ok(())
}

