let cli_args = match_args(["network", "id"]);

wasm::query(
    #{
        network: cli_args.network,
        contract_name: "portal",
        msg: #{ data: #{
            id: cli_args.id
        }}
    }
);
