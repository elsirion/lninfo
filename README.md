# Lightning Node Connection Tester

This tool connects to an LN node, syncs gossip and prints the node's info. Syncing the entire gossip set  is wasteful,
fixes welcome.

## Why?
I had misconfigured my firewall once too often and wanted a way to easily determine if my node was reachable. It was
also a good excuse to use [Lightning Dev Kit](https://lightningdevkit.org/).

## Usage

```
$ cargo run -- 
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/lninfo`
error: The following required arguments were not provided:
  <NODE_ID>
  <ADDR>

Usage: lninfo <NODE_ID> <ADDR>

For more information try '--help'
```