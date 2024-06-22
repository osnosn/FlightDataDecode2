# 安装rust编译环境，参考rust官网 https://www.rust-lang.org/tools/install
# 在debian中执行安装脚本: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 静态链接
#RUSTFLAGS='-C target-feature=+crt-static' cargo build --release
## RUSTFLAGS='-C target-feature=+crt-static -C link-args=-no-pie' cargo build --release

.PHONY: all debug release clean fmt cp 7z one7z onezip

help:
	@echo ' 可用的命令:'
	@echo '     make linux64'
	@echo '       编译x86_64-unknown-linux-gnu,静态链接'
	@echo '     make linux64musl'
	@echo '       编译x86_64-unknown-linux-musl,静态链接'
	@echo '     make win32'
	@echo '       编译i686-pc-windows-gnu'
	@echo '     make win64'
	@echo '       编译x86_64-pc-windows-gnu'
	@echo '     make arm64'
	@echo '       编译aarch64-unknown-linux-gnu,静态链接'
	@echo '  *  make all | a'
	@echo '       编译所有的target,静态链接'
	@echo '  *  make release | r'
	@echo '       编译default target,动态链接'
	@echo '     make debug | d'
	@echo '       编译default target,动态链接,--debug'
	@echo '  *  make fmt'
	@echo '       cargo fmt'
	@echo '     make clean'
	@echo '       cargo clean'
	@echo '  *  make cp'
	@echo '       把所有编译好的bin文件copy到"binfiles/"目录中'
	@echo '  *  make 7z'
	@echo '       把"binfiles/"目录中,按target目录,分别压缩为7z文件'
	@echo '     make one7z'
	@echo '       把"binfiles/"目录中所有文件,压缩为一个7z文件'
	@echo '     make onezip'
	@echo '       把"binfiles/"目录中所有文件,压缩为一个zip文件'

release r:
	@# 默认动态链接
	@echo target default
	@cargo build --release

linux64:
	@echo target x86_64-unknown-linux-gnu
	@RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-unknown-linux-gnu

linux64musl:
	@echo target x86_64-unknown-linux-musl
	@# mlua, luajit52,不支持 musl; lua54可以支持;
	@RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-unknown-linux-musl --workspace --exclude qar_decode_lua

win32:
	@echo target i686-pc-windows-gnu
	@# mlua, luajit52,不支持 win32; lua54可以支持;
	@RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target i686-pc-windows-gnu --workspace --exclude qar_decode_lua

win64:
	@echo target x86_64-pc-windows-gnu
	@RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-pc-windows-gnu

arm64:
	@echo target aarch64-unknown-linux-gnu
	@RUSTFLAGS='-C linker=aarch64-linux-gnu-gcc -C target-feature=+crt-static' cargo build --release --target aarch64-unknown-linux-gnu

all a: release linux64 linux64musl win32 win64 arm64

debug d:
	cargo build

clean:
	cargo clean

fmt:
	cargo fmt

#----------------------------------------
BINFILES := dump_raw_aligned dump_raw_bitstream dump_raw_bitstream2
#BINFILES += qar_decode qar_decode2 qar_decode3 qar_decode4 qar_decode5 qar_decode6 
BINFILES += qar_decode7 qar_decode8 qar_decode9 qar_datafile2
TARGETS1 := aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu x86_64-unknown-linux-musl
TARGETS2 := i686-pc-windows-gnu x86_64-pc-windows-gnu

cp:
	-@for target_single in $(TARGETS1); do \
		echo mkdir -p binfiles/$${target_single};\
		mkdir -p binfiles/$${target_single};\
		for binf in $(BINFILES); do \
#			echo cp target/$${target_single}/release/$${binf}  binfile/$${target_single}/$${binf};\
			cp target/$${target_single}/release/$${binf}  binfiles/$${target_single}/$${binf};\
		done \
	done
	-@for target_single in $(TARGETS2); do \
		echo mkdir binfiles/$${target_single};\
		mkdir binfiles/$${target_single};\
		for binf in $(BINFILES); do \
#			echo cp target/$${target_single}/release/$${binf}.exe  binfile/$${target_single}/$${binf}.exe; \
			cp target/$${target_single}/release/$${binf}.exe  binfiles/$${target_single}/$${binf}.exe; \
		done \
	done

7z:
	-@for target_single in $(TARGETS1) $(TARGETS2); do\
		rm binfiles/$${target_single}.7z;\
		echo 7z a binfiles/$${target_single}.7z  binfiles/$${target_single};\
		7z a binfiles/$${target_single}.7z  binfiles/$${target_single};\
	done

one7z:
	-@rm binfiles.7z
	@echo 7z a -xr'!*.7z' -xr'!*.zip' binfiles.7z  binfiles
	@7z a -xr'!*.7z' -xr'!*.zip' binfiles.7z  binfiles

onezip:
	-@rm binfiles.zip
	@echo 7z a -mx9 -xr'!*.7z' -xr'!*.zip' binfiles.zip  binfiles
	@7z a -mx9 -xr'!*.7z' -xr'!*.zip' binfiles.zip  binfiles

