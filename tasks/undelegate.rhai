// # Matching command line arguments
// matches the arguments passed to the script and returns a map of the arguments

let cli_args = match_args(["signer", "network", "id", "amount", "spender"]);

// with this setup, we can run commands like:
// $ beaker task run redeem -- --signer test1 --network localnet --amount 1 --id linkvaloper1twsfmuj28ndph54k4nw8crwu8h9c8mh33lyrp8/0 --spender link1u235cpgju5vvlzp4w53vu0z5x3etytdpeh78ffekctfcmfc8ezhsm6uhuh
wasm::execute(
    #{
        signer_account: cli_args.signer,
        network: cli_args.network,
        contract_name: "cw20_base",
        msg: #{
            increase_allowance: #{
                spender: cli_args.spender,
                amount: cli_args.amount
            }
        }
    }
);

wasm::execute(
    #{
        signer_account: cli_args.signer,
        network: cli_args.network,
        funds: cli_args.funds,
        contract_name: "portal",
        msg: #{
            undelegate: #{
                id: cli_args.id,
                amount: cli_args.amount
            }
        }
    }
);
