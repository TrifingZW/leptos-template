# 本地 TLS 证书

模板默认不启用 TLS。需要本地 HTTPS 调试时，在项目根目录执行：

```sh
openssl req -x509 -newkey rsa:2048 -nodes -days 3650 \
  -subj "/CN=leptos-template.local" \
  -addext "subjectAltName=DNS:leptos-template.local,DNS:localhost,IP:127.0.0.1,IP:::1" \
  -addext "basicConstraints=critical,CA:FALSE" \
  -addext "keyUsage=critical,digitalSignature,keyEncipherment" \
  -addext "extendedKeyUsage=serverAuth" \
  -keyout self_signed_certs/leptos-template.key \
  -out self_signed_certs/leptos-template.crt
```

生成的私钥已被 `.gitignore` 忽略，请勿提交到版本库。将 `config.toml` 中
`[server.tls].enabled` 设为 `true` 后重启服务。
