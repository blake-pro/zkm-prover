# README

## Description

The script file `gen_config.sh` allow you generate multi prover toml in a easy way.

First, you should set these variables according to your environment.

- provers
- stage
- proving_key_paths = ["/mnt/data/zkm2/proving.key"]
- tls
- base_dir

Notice that the `proving_key_paths` list now contains only the `prover_v2` proving key path.

Then you can run this script in below way.

```bash
bash gen_config.sh
```
