# Primeiros passos

[English](getting-started.md) · Português

Este guia leva você do zero até o DiskMarshal instalado no seu Mac e aberto, um passo de cada vez. Não é preciso saber programar.

- **Tempo:** cerca de 15 minutos, a maior parte esperando downloads e a compilação.
- **Espaço em disco:** deixe uns 6 GB livres para as ferramentas e a compilação. O app instalado ocupa cerca de 10 MB.
- **Testado em:** macOS 27 em Apple Silicon. Outras versões e Macs com Intel devem funcionar, mas ainda não foram testados. Se algo falhar, [avise a gente](https://github.com/renatoasterio/DiskMarshal/issues).

> Instalar e abrir o DiskMarshal não apaga nada. O app só remove o que você seleciona, depois de mostrar uma prévia e pedir confirmação. Leia o [guia de segurança](safety.pt-BR.md) antes da primeira limpeza: ele também diz com clareza o que ainda não foi testado.

## Como seguir este guia

Cada caixa cinza é um comando. Para cada uma:

1. Copie o comando.
2. Cole no Terminal.
3. Pressione Return e espere o Terminal voltar a aceitar digitação.

Para abrir o Terminal, pressione `Command + Espaço`, digite `Terminal` e pressione Return.

Você só precisa do Terminal para instalar. Depois disso, o DiskMarshal abre como qualquer outro app.

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

O DiskMarshal é escrito em Rust, então você precisa das ferramentas do Rust para compilá-lo.

> Já tem o Rust (`cargo --version` mostra uma versão)? Pule para o passo 4.

Instale o instalador do Rust:

```bash
brew install rustup
```

O Homebrew guarda as ferramentas do Rust em uma pasta própria (`/opt/homebrew/opt/rustup/bin` em Apple Silicon), onde o Terminal não procura sozinho. Diga ao Terminal onde encontrá-las:

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

Deve aparecer algo como `cargo 1.100.0`. O projeto fixa a versão exata do Rust de que precisa (no arquivo `rust-toolchain.toml`) e a baixa sozinho na primeira compilação.

## Passo 4. Baixe o DiskMarshal

Escolha onde a pasta do projeto vai ficar. Este exemplo usa a sua pasta pessoal.

```bash
cd ~
```

```bash
git clone https://github.com/renatoasterio/DiskMarshal.git
```

Entre na pasta. Todos os comandos daqui em diante precisam rodar dentro dela.

```bash
cd DiskMarshal
```

## Passo 5. Compile e instale o app

```bash
cargo xtask install
```

Esse único comando faz quatro coisas:

1. Compila o DiskMarshal. Na primeira vez, também baixa as bibliotecas que o DiskMarshal usa. Conte com alguns minutos. Linhas começando com `Compiling` são normais.
2. Monta o app em `dist/DiskMarshal.app`, dentro da pasta do projeto: o app em si, um pequeno auxiliar usado só na limpeza de sistema e o ícone.
3. Assina o app, para o macOS aceitar abri-lo.
4. Copia para `/Applications`, a pasta Aplicativos do seu Mac.

**Confira se deu certo:** as últimas linhas devem ser estas.

```text
xtask: installed /Applications/DiskMarshal.app
xtask: open it from Launchpad, Spotlight or the Applications folder
```

Se a cópia falhar porque a sua conta não pode gravar na pasta Aplicativos, instale o app na pasta Aplicativos do seu próprio usuário:

```bash
cargo xtask install ~/Applications
```

## Passo 6. Abra o app

Pressione `Command + Espaço`, digite `DiskMarshal` e pressione Return. Você também pode abrir pelo Launchpad ou pela pasta Aplicativos no Finder.

**O que você deve ver:** uma sequência curta de abertura (clique para pular) e, em seguida, o "Painel".

![O painel do DiskMarshal: barra de status em cima, as dez telas à esquerda, medidores e o espaço recuperável à direita](../assets/screens/dashboard.png)

*Captura do app com dados de demonstração. Os seus números serão outros, e "Espaço recuperável" começa com "não analisado" em todas as linhas.*

Pressione `1` a `7` para visitar cada tela. Para sair, pressione `Command + Q`.

## Passo 7. Dê o Acesso Total ao Disco (opcional)

O macOS mantém algumas pastas, como os dados do Mail e do Safari e a Lixeira, escondidas de todo app até você permitir. O DiskMarshal funciona sem essa permissão; ele só não consegue ver nem medir essas pastas.

Quando a permissão falta, as telas que leem as suas pastas mostram esta linha no topo:

> Sem Acesso Total ao Disco, o macOS esconde parte da sua pasta pessoal das análises. **Abrir ajustes de Acesso Total ao Disco**

Para dar a permissão:

1. Clique no link **"Abrir ajustes de Acesso Total ao Disco"**. Os Ajustes do Sistema abrem na página certa.
2. Ligue a chave ao lado de **DiskMarshal**. Se o DiskMarshal não estiver na lista, clique no botão **+** e escolha o app na pasta Aplicativos.
3. Se o macOS pedir a sua senha ou o Touch ID para confirmar a mudança, informe.
4. Encerre o DiskMarshal (`Command + Q`) e abra de novo.

Você pode desligar a chave do mesmo jeito quando quiser.

> **O macOS esquece essa permissão quando o app é recompilado.** O app que você acabou de compilar leva uma assinatura local (chamada ad hoc), que muda a cada compilação. Toda vez que você roda `cargo xtask install` de novo, o macOS trata o resultado como um app novo, e é preciso repetir este passo. Assinar com uma identidade sua evita isso: defina a variável de ambiente `DISKMARSHAL_SIGNING_IDENTITY` com o nome da identidade antes de rodar `cargo xtask install`.

## Passo 8. Meça uma pasta pelo Terminal (opcional)

O DiskMarshal também tem uma pequena ferramenta de linha de comando. Ela mede uma pasta e lista os maiores itens, sem alterar nada:

```bash
cargo run --release -p diskmarshal-cli -- scan ~/Downloads
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

O [guia de uso](usage.pt-BR.md#a-ferramenta-de-linha-de-comando) explica cada linha.

## Pronto

| Você quer | Faça isto |
|---|---|
| Abrir o app | Launchpad ou Spotlight, como qualquer outro app. O Terminal não é necessário. |
| Entender o que cada tela faz | Leia o [guia de uso](usage.pt-BR.md). |
| Pegar a versão mais nova | `cd ~/DiskMarshal`, depois `git pull`, depois `cargo xtask install` |
| Instalar em outro lugar | `cargo xtask install ~/Applications` (ou qualquer outra pasta) |
| Só montar o app, sem instalar | `cargo xtask bundle`. O resultado é o `dist/DiskMarshal.app`, dentro da pasta do projeto. |
| Abrir o app sem instalar | `cargo run --release -p diskmarshal`. O grupo "Sistema" da tela de limpeza pode ficar indisponível desse jeito: veja o [guia de uso](usage.pt-BR.md#o-grupo-sistema). |
| Liberar o espaço usado pela compilação | `cd ~/DiskMarshal` e depois `cargo clean`. O app instalado continua funcionando. |
| Remover o DiskMarshal | Mova o `/Applications/DiskMarshal.app` para a Lixeira. Depois, se quiser, apague o histórico de remoções em `~/Library/Application Support/DiskMarshal/` e a pasta `DiskMarshal` do projeto. |

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
