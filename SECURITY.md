# Segurança

Para reportar uma falha de segurança, **não abra uma issue pública**. Use o
botão "Report a vulnerability" na aba
[Security](https://github.com/yurisanp/cuvee/security/advisories/new) do
repositório. Responda com o que encontrou, como reproduzir e qual versão.

As releases são assinadas com [minisign](https://jedisct1.github.io/minisign/).
A chave pública fica em `cuvee.pub`, na raiz do repositório. Para conferir um
arquivo baixado:

```sh
minisign -Vm cuvee-gui-vX.Y.Z-x86_64-linux -p cuvee.pub
```
