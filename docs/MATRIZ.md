# Matriz de testes

Critério da v1.0: cada combinação de GPU do Linux, sessão gráfica e versão do
macOS foi testada por alguém em hardware real. Os resultados chegam pelo modelo
de issue **Resultado de teste (matriz)**, com o arquivo do `cuvee report`.

Legenda: **ok** funciona bem · **limite** funciona com limites (veja a issue) ·
**falha** não funciona · **?** ainda não testado.

| macOS | AMD · Wayland | AMD · X11 | Intel · Wayland | Intel · X11 | NVIDIA · Wayland | NVIDIA · X11 |
|---|---|---|---|---|---|---|
| 26 Tahoe | ? | ? | ? | ? | ? | ? |
| 15 Sequoia | ? | ? | ? | ? | ? | ? |
| 14 Sonoma | ? | ? | ? | ? | ? | ? |
| 13 Ventura | ? | ? | ? | ? | ? | ? |
| 12 Monterey | ? | ? | ? | ? | ? | ? |
| 11 Big Sur | ? | ? | ? | ? | ? | ? |
| 10.15 Catalina | ? | ? | ? | ? | ? | ? |
| 10.14 Mojave | ? | ? | ? | ? | ? | ? |
| 10.13 High Sierra | ? | ? | ? | ? | ? | ? |

Sem passthrough, só o macOS 26 tem aceleração gráfica; nas outras versões,
"ok" quer dizer que instala, liga e o desktop é usável sem Metal. Com GPU
passthrough (v0.4), registre o resultado na issue e cite a placa passada.

Cada célula preenchida leva o número da issue, por exemplo `ok #12`.
