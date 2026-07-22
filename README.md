# Seolgi -- 설기

설기, n. 싸리채나 버들 채 따위로 엮어서 만든 네모꼴의 상자. 아래위 두 짝으로 되어
위짝으로 아래짝을 엎어 덮게 되어 있다. (표준국어대사전)

NOTE: Alpha quality software. Stability of the configuration file is NOT guaranteed.

## Project goal
Seolgi is a sandbox app launcher which aims to provide a secure and lightweight
isolated environment between the rest of the system and app.

## Backends
Currently, it only supports the `landlock` backend, which uses stackable `landlock`
LSM to provide filesystem level isolation.

## Configuration example
Refer `config.yaml.example`

## Install
Run

```shell
git clone https://github.com/perillamint/seolgi.git
cd seolgi
cargo install --path .
mkdir -p ~/.config/seolgi
cp config.yaml.example ~/.config/seolgi/config.yaml
```
