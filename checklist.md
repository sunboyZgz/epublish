<!--
 * @Author: sunboy
 * @LastEditors: sunboy
 * @Date: 2022-06-12 20:23:23
 * @LastEditTime: 2022-06-12 20:38:29
-->
## features in future
- [ ] this isn't a usable feature
you can publish -u patch, if you want publish your crate to crate.io after increasing the version
```shell
cargo epublish publish -u patch
```
I want to achieve this feature, but every time before your release cargo will warn that it should `git add .`, but I can't sure every one need perform this command
- [ ] need to capture stdout of cargo and git and make a perfect trace
