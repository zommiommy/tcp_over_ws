# tcp_over_ws
Small utility to tunel a tcp connection through websockets, this can be useful to bypass firewalls or exploit proxies that only allows http/https traffic.

This tunnel adds its own layer of encryption based on the `crypto_box` crate.

You can find pre-compiled versions of the server and client in the releases on Github.

My most common use-case is to forward SSH on a server that has a proxied
Cloudflare DNS Record. This might result in higher latency, maybe because
the connection is encrypted 3 times, https - my encryption - ssh tls.

# Example
Suppose that I want to access the Ip `192.168.1.10` on port `80` on the local
network of the server.

On the server machine we start a web socket listener on port `1987`:
```
./server --port 1987 --target-ip 192.168.1.10 --target-port 80
```

And then on the client machine we can connect to the server with:
```
./client --target 'ws://$SERVER_IP:1987' --port 1989
```

Now we can connect to `127.0.0.1:1989` to access `192.168.1.10:80`.
