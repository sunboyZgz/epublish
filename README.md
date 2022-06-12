<!--
 * @Author: sunboy
 * @LastEditors: sunboy
 * @Date: 2022-06-12 19:59:09
 * @LastEditTime: 2022-06-12 20:33:50
-->
## reference
this tool obeys the semantic in the link [https://semver.org/](https://semver.org/).
## Note
this binary executable file is only compatible with windows, if you run it on the linux, some unexpected errors may stick you. Besides, it's only a newbie's exercise demo.
#### install
```shell
cargo install cargo-epublish
```
#### usage
add a section like below pieces into Cargo.toml or customized toml configuration file
```toml
[max-versions]
minor = 10
patch = 10
```
you can [patch/minor/major] your crate version
```shell
cargo epublish patch
// the same as 
cargo epublish -u patch
```

