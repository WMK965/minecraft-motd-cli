# motd-cli
像 PING 一样获取我的世界服务器 MOTD 信息

由 [965](https://github.com/WMK965) 基于 [BlackBEDevelopment/motd-cli](https://github.com/BlackBEDevelopment/motd-cli) 使用 Rust 重构，添加 verbose 和 help 模式

## 🛫 如何使用
前往 [releases](https://github.com/WMK965/minecraft-motd-cli/releases) 下载最新版本解压即可
```
# motd-cli
motd-cli - 像 PING 一样获取我的世界服务器 MOTD 信息

用法:
  motd-cli <be|je> <地址[:端口]>    指定服务器类型查询
  motd-cli <地址[:端口]>            自动推断服务器类型

参数:
  be                 基岩版服务器 (默认端口: 19132)
  je                 Java版服务器 (默认端口: 25565)

选项:
  -h, --help         显示此帮助信息
  -v, --verbose      显示详细调试信息
  -j, --json         以 JSON 格式输出结果

示例:
  motd-cli be play.craftersmc.net:19132
  motd-cli je play.hypixel.net
  motd-cli play.craftersmc.net
  motd-cli -v be play.craftersmc.net

# motd-cli je play.hypixel.net
正在获取 play.hypixel.net:25565 [172.65.197.160:25565] 的 MOTD 信息...

  延迟:      196ms
  在线人数:  14838/200000
  协议版本:  575
  游戏版本:  Requires MC 1.8 / 1.21
  MOTD:                       Hypixel Network [1.8/1.21]
       EASTER EVENT + ANNIVERSARY BINGO

# motd-cli -j play.craftersmc.net
{"status":"online","host":"play.craftersmc.net:19132","motd":"CraftersMC ✧ SkyBlock: End Beyond!","agreement":944,"version":"1.26.10","online":270,"max":1000,"level_name":"Play Now!","game_mode":"Survival","server_unique_id":"5800216507803989860","delay":276,"type":"Bedrock Edition"}
```

## ⚙️ 构建程序
依赖： Rust >= 1.70
### 克隆仓库
```
git clone https://github.com/WMK965/minecraft-motd-cli.git
```
### 编译项目(Windows)
``` shell
cargo build --release
```
编译产物位于 `target/release/motd-cli.exe`
### 编译项目(Linux)
``` shell
cargo build --release
```
编译产物位于 `target/release/motd-cli`

## 🔗 设置为环境变量后使用
为了获得更好的体验，你可以尝试设置环境变量来通过如下方式在命令行使用程序
### Windows
1. 将下载到的文件解压到单独的文件夹内，记住文件夹的路径
2. 打开`高级系统设置`
3. 在高级选项卡选择环境变量
4. `用户环境变量`选项框内找到`Path`选项
5. 打开`Path`选项，选择`新建`，然后选择`浏览`后选择刚刚解压的程序的文件夹
6. 点击`确定`后重启电脑即可使用
### Linux（针对BASH）
1. 将下载到的文件解压到单独的文件夹内，记住文件夹的路径
2. 输入`vim ~/.bashrc`打开环境变量配置
3. 在最后一行加上`export PATH=$PATH:{{path}}` {{path}}为刚刚存放程序的文件夹路径
4. 保存文件后输入`source ~/.bashrc`使配置文件生效即可
