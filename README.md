# lane - A command to quickly show/set/clear proxies/mirrors for different programs

## Supported apps

* curl: proxy
* git: proxy
* cargo: proxy, mirror(tuna/ustc)

## Usage

Some examples:

```shell
# show all proxies of supported apps
lane show-proxy
# set proxy of curl only
lane set-proxy curl -p http://127.0.0.1:8080
# clear proxy to all supported apps
lane clear-proxy
# set mirror of cargo to tuna
lane set-mirror cargo tuna
```