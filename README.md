# hash box

用于增量同步文件

## 原理

...

## 安装

```bash
cargo install --path ./
```

## 用法

```bash
hbx --help
```

## 交叉编译

利用musl库代替glibc

```bash
docker run --rm -it -v "$(pwd)":/home/rust/src ekidd/rust-musl-builder cargo build --release
```

## todo

- p2p方式批量同步

## License

Apache-2.0