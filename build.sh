
 # 安装rust编译环境，参考rust官网 https://www.rust-lang.org/tools/install
 # 在debian中执行安装脚本: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

 # 静态链接
 #RUSTFLAGS='-C target-feature=+crt-static' cargo build --release
 ## RUSTFLAGS='-C target-feature=+crt-static -C link-args=-no-pie' cargo build --release

 # 默认动态链接
 echo target default
 cargo build --release

 if [ "$1" = "all" ]; then
   #其他
   echo target x86_64-unknown-linux-gnu
   RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-unknown-linux-gnu
   echo target x86_64-unknown-linux-musl
   # mlua, luajit52,不支持 musl; lua54可以支持;
   RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-unknown-linux-musl --workspace --exclude qar_decode_lua
   echo target i686-pc-windows-gnu
   # mlua, luajit52,不支持 win32; lua54可以支持;
   RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target i686-pc-windows-gnu --workspace --exclude qar_decode_lua
   echo target x86_64-pc-windows-gnu
   RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-pc-windows-gnu
   echo target aarch64-unknown-linux-gnu
   RUSTFLAGS='-C linker=aarch64-linux-gnu-gcc -C target-feature=+crt-static' cargo build --release --target aarch64-unknown-linux-gnu
 else
	 echo
	 echo "  交叉编译 静态链接的 5个目标平台的执行文件:"
	 echo "      $0 all"
	 echo
 fi

