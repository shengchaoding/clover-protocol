use serde_json::{json, Value};

use ckb_types::core::TransactionBuilder;

const NODE_RPC_ADDR: &'static str = "http://127.0.0.1:8114";

fn jsonrpc(method: &str, params: Vec<&str>) -> Value {
    json!(
        {
            "id": 0,
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        }
    )
}

pub struct Tx;

pub async fn query_block_deposits(block_height: u64) -> Result<(Vec<Tx>, u64), ()> {
    //get_tip_block_number
    let now_height = match surf::post(NODE_RPC_ADDR)
        .body_json(&jsonrpc("get_tip_block_number", vec![]))
        .map_err(|_e| ())?
        .await
    {
        Ok(mut res) => {
            if let Ok(mut value) = res.body_json::<Value>().await {
                let result = value["result"].take();
                let hex_num = result.as_str().ok_or(())?;
                u64::from_str_radix(&hex_num[2..], 16).map_err(|_| ())?
            } else {
                return Err(());
            }
        }
        Err(err) => return Err(()),
    };

    println!("now_height: {:?}", now_height);

    if now_height <= block_height {
        return Ok((vec![], block_height));
    }

    let mut deposit_txs = vec![];
    let mut change_block_height = block_height;

    for i in block_height..now_height {
        // get block hash
        if let Ok(mut res) = surf::post(NODE_RPC_ADDR)
            .body_json(&jsonrpc("get_header_by_number", vec![&format!("{:#x}", i)]))
            .map_err(|_e| ())?
            .await
        {
            let result = res.body_json::<Value>().await.map_err(|_| ())?;
            let hash = result["result"]["hash"].as_str().ok_or(())?;

            // get block info
            if let Ok(mut res) = surf::post(NODE_RPC_ADDR)
                .body_json(&jsonrpc("get_block", vec![hash]))
                .map_err(|_e| ())?
                .await
            {
                let result = res.body_json::<Value>().await.map_err(|_| ())?;
                let transactions = result["result"]["transactions"].as_array().ok_or(())?;

                for tx in transactions {
                    println!("{:?}", tx);
                    // TODO CHECK Tx is to our contract.
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }

    Ok((deposit_txs, block_height))
}

pub async fn post_block() -> Result<String, ()> {
    let tx = json!(
          {
            "cell_deps": [
                {
                    "dep_type": "code",
                    "out_point": {
                        "index": "0x0",
                        "tx_hash": "0xa4037a893eb48e18ed4ef61034ce26eba9c585f15c9cee102ae58505565eccc3"
                    }
                }
            ],
            "header_deps": [
                "0x7978ec7ce5b507cfb52e149e36b1a23f6062ed150503c85bbf825da3599095ed"
            ],
            "inputs": [
                {
                    "previous_output": {
                        "index": "0x0",
                        "tx_hash": "0x365698b50ca0da75dca2c87f9e7b563811d3b5813736b8cc62cc3b106faceb17"
                    },
                    "since": "0x0"
                }
            ],
            "outputs": [
                {
                    "capacity": "0x2540be400",
                    "lock": {
                        "args": "0x",
                        "code_hash": "0x28e83a1277d48add8e72fadaa9248559e1b632bab2bd60b27955ebc4c03800a5",
                        "hash_type": "data"
                    },
                    "type": null
                }
            ],
            "outputs_data": [
                "0x"
            ],
            "version": "0x0",
            "witnesses": []
        }
    );

    // Build a CKB Transaction
    let rpc_call = json!(
        {
            "id": 0,
            "jsonrpc": "2.0",
            "method": "send_transaction",
            "params": [tx],
        }
    );

    // NODE RPC send_transaction
    match surf::post(NODE_RPC_ADDR)
        .body_json(&rpc_call)
        .map_err(|_e| ())?
        .await
    {
        Ok(mut res) => match res.body_json::<Value>().await {
            Ok(mut value) => {
                println!("{:?}", value);
                let tx_id = value["result"].as_str().ok_or(())?;
                println!("tx_id: {:?}", tx_id);
                return Ok(tx_id.to_owned());
            }
            Err(err) => {
                println!("{:?}", err);
                Err(())
            }
        },
        Err(err) => {
            println!("{:?}", err);
            Err(())
        }
    }
}

//
// Transaction:
// {
//     "cell_deps": [],
//     "hash": "0x365698b50ca0da75dca2c87f9e7b563811d3b5813736b8cc62cc3b106faceb17",
//     "header_deps": [],
//     "inputs": [
//         {
//             "previous_output": {
//                 "index": "0xffffffff",
//                 "tx_hash": "0x0000000000000000000000000000000000..."
//             },
//             "since": "0x400"
//         }
//     ],
//     "outputs": [
//         {
//             "capacity": "0x18e64b61cf",
//             "lock": {
//                 "args": "0x",
//                 "code_hash": "0x28e83a1277d48add8e72fadaa9248559e1b...bc4c03800a5",
//                 "hash_type": "data"
//             },
//             "type": null
//         }
//     ],
//     "outputs_data": [
//         "0x"
//     ],
//     "version": "0x0",
//     "witnesses": [
//         "0x450000000c0000004100000...c4c03800a5000000000000000000"
//     ]
// }