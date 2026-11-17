# Primeiros passos

[English](getting-started.md) · Português

Este guia leva você do zero até a janela do MacCleaner aberta no seu Mac, um passo de cada vez. Não é preciso saber programar.

- **Tempo:** cerca de 15 minutos, a maior parte esperando downloads.
- **Espaço em disco:** deixe uns 6 GB livres para as ferramentas e a compilação.
- **Testado em:** macOS 27 em Apple Silicon. Outras versões e Macs com Intel devem funcionar, mas ainda não foram testados. Se algo falhar, [avise a gente](https://github.com/renatoasterio/MacCleaner/issues).

> O MacCleaner está em desenvolvimento inicial. Seguir este guia é seguro: hoje o app e a ferramenta de linha de comando só leem o disco. Nada é apagado.

## Como seguir este guia

Cada caixa cinza é um comando. Para cada uma:

1. Copie o comando.
2. Cole no Terminal.
3. Pressione Return e espere o Terminal voltar a aceitar digitação.

Para abrir o Terminal, pressione `Command + Espaço`, digite `Terminal` e pressione Return.

## Passo 1. Instale as ferramentas de linha de comando do Xcode

São as ferramentas básicas da Apple para compilar programas. O Git vem junto.

```bash
xcode-select --install
```

Uma janela pede confirmação. Clique em **Instalar** e espere terminar.

**Confira se deu certo:**

```bash
xcode-select -p
```

Deve aparecer um caminho, como `/Library/Developer/CommandLineTools`.

Se o primeiro comando respondeu `command line tools are already installed`, você já tem as ferramentas. Siga para o próximo passo.

## Passo 2. Instale o Homebrew

O Homebrew instala ferramentas de desenvolvimento no Mac. Primeiro veja se você já tem:

```bash
brew --version
```

- Se aparecer um número de versão, como `Homebrew 5.0.0`, vá para o passo 3.
- Se aparecer `command not found`, abra o site [brew.sh](https://brew.sh), copie o comando de instalação que aparece no topo da página, cole no Terminal e pressione Return. Ao terminar, ele mostra uma seção chamada **Next steps** com dois ou três comandos. Rode esses também e depois rode `brew --version` de novo.

## Passo 3. Instale o Rust

O MacCleaner é escrito em Rust, então você precisa das ferramentas do Rust para compilá-lo.

> Já tem o Rust (`cargo --version` mostra uma versão)? Pule para o passo 4.

Instale o instalador do Rust:

```bash
brew install rustup
```

Diga ao Terminal onde encontrar as ferramentas do Rust:

```bash
echo "export PATH=\"$(brew --prefix rustup)/bin:\$PATH\"" >> ~/.zshrc
```

Recarregue as configurações do Terminal:

```bash
source ~/.zshrc
```

Instale o Rust em si:

```bash
rustup default stable
```

**Confira se deu certo:**

```bash
cargo --version
```

Deve aparecer algo como `cargo 1.100.0`. O projeto fixa a versão exata do Rust de que precisa e a baixa sozinho na primeira compilação.

## Passo 4. Baixe o MacCleaner

Escolha onde a pasta do projeto vai ficar. Este exemplo usa a sua pasta pessoal.

```bash
cd ~
```

```bash
git clone https://github.com/renatoasterio/MacCleaner.git
```

Entre na pasta. Todos os comandos daqui em diante precisam rodar dentro dela.

```bash
cd MacCleaner
```

## Passo 5. Abra o app

```bash
cargo run --release -p maccleaner
```

Na primeira vez, isso baixa as bibliotecas que o MacCleaner usa e compila tudo. Conte com cerca de um minuto em um Mac recente e mais em máquinas antigas. Linhas começando com `Compiling` são normais.

**O que você deve ver:** uma sequência curta de abertura e, em seguida, esta janela.

![A janela do MacCleaner: barra de status em cima, trilho de navegação à esquerda, área de trabalho à direita](../assets/app-current.png)

Pressione `1` a `7` para visitar cada tela. Para sair, pressione `Command + Q`.

As telas ainda mostram "Módulo em construção". Isso é esperado: veja o [estado do projeto](../../README.pt-BR.md#estado-do-projeto).

## Passo 6. Meça uma pasta

A ferramenta de linha de comando já faz trabalho de verdade. Ela mede uma pasta e lista os maiores itens, sem alterar nada:

```bash
cargo run --release -p maccleaner-cli -- scan ~/Downloads
```

O macOS pode perguntar se o Terminal pode acessar a pasta Downloads. Clique em **Permitir**.

**O que você deve ver:** um resumo como este, com os seus números.

```text
Pasta              /Users/voce/Downloads
Itens              2.418
Ocupado em disco   27,1 GB
Tamanho lógico     27,0 GB
Somente na nuvem   0 B
Pastas sem acesso  0
Tempo              0,1 s

Maiores itens
     12,0 GB  Instaladores/
      9,2 GB  backup-2023.zip
```

O [guia de uso](usage.pt-BR.md) explica cada linha.

## Pronto

| Você quer | Faça isto |
|---|---|
| Abrir o app de novo | `cd ~/MacCleaner` e depois `cargo run --release -p maccleaner` |
| Pegar a versão mais nova | `cd ~/MacCleaner`, depois `git pull`, depois abra o app de novo |
| Liberar o espaço usado pela compilação | `cd ~/MacCleaner` e depois `cargo clean` |
| Remover o MacCleaner | Mova a pasta `MacCleaner` para a Lixeira |

## Se algo der errado

A página de [perguntas e solução de problemas](faq.pt-BR.md) lista as mensagens de erro mais comuns e como resolver cada uma.

## Para quem vai contribuir

Para rodar as mesmas verificações da integração contínua, instale o auditor de dependências uma vez e rode o portão de qualidade:

```bash
brew install cargo-deny
```

```bash
cargo xtask check
```

Depois leia o [CONTRIBUTING.md](../../CONTRIBUTING.md).
