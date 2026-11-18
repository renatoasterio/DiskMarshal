<div align="center">

<img src="docs/assets/app-icon.png" alt="O ícone do app MacCleaner" width="112">

# MacCleaner

**Limpador de disco e monitor do sistema para macOS, de código aberto, escrito em Rust.**

[English](README.md) · Português

[![CI](https://github.com/renatoasterio/MacCleaner/actions/workflows/ci.yml/badge.svg)](https://github.com/renatoasterio/MacCleaner/actions/workflows/ci.yml)
[![Licença: MIT](https://img.shields.io/badge/licen%C3%A7a-MIT-2fe6ff)](LICENSE)
![Plataforma: macOS](https://img.shields.io/badge/plataforma-macOS-2fe6ff)
![Rust 1.100](https://img.shields.io/badge/rust-1.100-2fe6ff)
![Estado: pré-lançamento](https://img.shields.io/badge/estado-pr%C3%A9--lan%C3%A7amento-ffb020)

<img src="docs/assets/tour.gif" alt="Sequência com as sete telas do MacCleaner: painel, limpeza, duplicados, fotos parecidas, desenvolvedor, mapa do disco e monitor" width="900">

<sub>Capturas do próprio app, rodando com dados de demonstração. Veja em <a href="#estado-do-projeto">Estado do projeto</a> o que está pronto e o que ainda não foi verificado.</sub>

</div>

## O que é

O MacCleaner descobre o que está enchendo o disco do seu Mac e ajuda a recuperar o espaço, com segurança.

- **Veja para onde foi o espaço.** Um mapa do disco, mostrando o que está neste Mac e o que está só no iCloud.
- **Remova o que é seguro remover.** Caches, arquivos temporários, logs, arquivos duplicados, fotos parecidas e as pastas de build que projetos de código deixam para trás (`node_modules`, `target`, `bin`/`obj` e companhia).
- **Acompanhe a máquina.** Processador, memória, disco e temperaturas, ao vivo.

Ele é construído em torno de uma regra: **nada é apagado sem prévia e sem a sua confirmação**, e arquivos pessoais sempre vão para a Lixeira.

## Instalar e abrir

Você precisa de um Mac com as ferramentas de linha de comando do Xcode, o Homebrew e o Rust. O [guia de primeiros passos](docs/guide/getting-started.pt-BR.md) explica cada etapa, inclusive como instalar esses três. Não é preciso saber programar.

Baixe o código:

```bash
git clone https://github.com/renatoasterio/MacCleaner.git
```

```bash
cd MacCleaner
```

Compile o app e instale:

```bash
cargo xtask install
```

Esse comando compila o app, monta o `dist/MacCleaner.app` (o app, o auxiliar da limpeza de sistema e o ícone), assina e copia para `/Applications` (a pasta Aplicativos). A primeira compilação baixa as dependências e leva alguns minutos.

Daí em diante, **o MacCleaner abre pelo Launchpad ou pelo Spotlight, como qualquer outro app**. A pasta do código só é necessária para alterar o código ou para atualizar.

| Comando | O que faz |
|---|---|
| `cargo xtask install` | Compila o app e instala em `/Applications` |
| `cargo xtask install ~/Applications` | Instala em outra pasta |
| `cargo xtask bundle` | Só monta o `dist/MacCleaner.app`, sem instalar |
| `cargo run --release -p maccleaner` | Abre o app direto do código, sem instalar |
| `cargo run --release -p maccleaner-cli -- scan ~/Downloads` | Mede uma pasta pelo terminal, sem alterar nada |

Três coisas que vale saber:

- **O Acesso Total ao Disco é opcional.** Sem ele o app funciona, mas o macOS esconde algumas pastas dele. As telas que leem as suas pastas mostram um aviso de uma linha com um link para o lugar certo nos Ajustes do Sistema.
- **A assinatura é local (ad hoc) por padrão.** Com ela, o macOS esquece a permissão de Acesso Total ao Disco toda vez que o app é recompilado e reinstalado. Para assinar com a sua própria identidade, defina a variável de ambiente `MACCLEANER_SIGNING_IDENTITY` com o nome dela antes de rodar `cargo xtask install`.
- **Para desinstalar,** mova o `/Applications/MacCleaner.app` para a Lixeira. O histórico de remoções fica em `~/Library/Application Support/MacCleaner/`; apague essa pasta também se não quiser guardá-lo.

## Estado do projeto

As sete telas estão prontas, e o app apaga de verdade o que você seleciona, sempre depois de uma prévia e de uma confirmação. Ainda não há uma versão estável. Ele é desenvolvido e testado em um único tipo de máquina: macOS 27 em Apple Silicon.

| Parte | O que faz | Estado |
|---|---|---|
| "Painel" | Medidores de disco, memória e temperatura, o espaço que as outras telas encontraram, as últimas remoções | Pronto |
| "Limpeza" | Caches, temporários, logs, a Lixeira, sobras do Xcode e de gerenciadores de pacotes | Pronto |
| "Limpeza", grupo "Sistema" | `/Library/Caches` e `/Library/Logs`, depois que o macOS autentica um administrador | Pronto, não verificado de ponta a ponta (veja abaixo) |
| "Duplicados" | Arquivos com conteúdo idêntico; as cópias a mais vão para a Lixeira | Pronto |
| "Fotos" | Imagens soltas em pastas que parecem a mesma; as que sobram vão para a Lixeira | Pronto |
| "Desenvolvedor" | Pastas de build e de dependências de projetos de código | Pronto |
| "Mapa do disco" | Mapa em blocos da pasta pessoal ou do disco inteiro, colorido por tipo ou por estado no iCloud | Pronto |
| "Monitor" | Processador, memória, swap, disco, temperaturas e os programas mais ativos, ao vivo | Pronto |
| `maccleaner-cli scan` | Mede uma pasta pelo terminal; somente leitura | Pronto |
| `cargo xtask install` | Compila, assina e instala o `MacCleaner.app` | Pronto |

### Ainda não verificado

Estas partes estão escritas e cobertas por testes automáticos em pastas temporárias, mas não foram confirmadas na situação real. Leve isso em conta.

- **Limpeza de sistema, de ponta a ponta.** O diálogo de autenticação do macOS e uma limpeza real de `/Library/Caches` e `/Library/Logs` não foram exercitados pelos testes do projeto. O auxiliar que faz a limpeza é coberto por testes em pastas temporárias e pelo seu modo de simulação (`--dry-run`). Se o diálogo oferece o Touch ID ou só a senha, quem decide é o macOS.
- **Um arquivo que está realmente só na nuvem.** A proteção contra downloads do iCloud não foi testada com um arquivo que esteja de fato só na nuvem.
- **Abrir com movimento reduzido.** O app lê a opção "Reduzir movimento" dos Ajustes do Sistema ao abrir; isso não foi testado com a opção ligada.
- **Apagar dados pessoais reais pela interface.** A remoção foi exercitada apenas em pastas de teste.
- **Outras versões do macOS e Macs com Intel.**

### Limites conhecidos

- Caminhos protegidos pelo macOS (Proteção da Integridade do Sistema, SIP) não podem ser limpos.
- Snapshots locais do Time Machine, `/private/var/log` e `/private/var/tmp` não são limpos.
- O monitor não mostra a velocidade das ventoinhas.
- Fotos dentro da biblioteca do app Fotos ficam de fora: a biblioteca nunca é lida nem alterada.
- Clones e snapshots do APFS podem fazer o espaço realmente liberado ser menor que o estimado.
- A interface existe só em português do Brasil, e as escolhas de aparência não são salvas entre uma abertura e outra.

## Telas

Capturas do app com dados de demonstração. O [guia de uso](docs/guide/usage.pt-BR.md) explica cada tela passo a passo.

| Painel | Limpeza |
|---|---|
| ![Painel com medidores de disco, memória e temperatura, o espaço recuperável por origem e as últimas remoções](docs/assets/screens/dashboard.png) | ![Lista de limpeza com uma etiqueta de risco em cada regra e um grupo separado para as áreas de sistema](docs/assets/screens/cleanup.png) |
| **Duplicados** | **Fotos parecidas** |
| ![Grupos de arquivos idênticos com a cópia a manter marcada](docs/assets/screens/duplicates.png) | ![Grupos de fotos parecidas com miniaturas, uma foto de referência e o percentual de correspondência](docs/assets/screens/photos.png) |
| **Desenvolvedor** | **Mapa do disco** |
| ![Pastas de build e de dependências listadas por projeto e ecossistema](docs/assets/screens/developer.png) | ![Mapa em que cada bloco tem o tamanho do espaço que a pasta ocupa](docs/assets/screens/disk-map.png) |
| **Monitor** | |
| ![Traço do processador, barras por núcleo, memória, disco, temperaturas e os programas mais ativos](docs/assets/screens/monitor.png) | |

Três paletas, escolhidas pelo ícone de aparência na barra de status:

![O painel nas paletas Agência, Terminal e Tático](docs/assets/palettes.png)

## Segurança desde o projeto

- **Prévia primeiro.** Toda remoção lista exatamente o que vai sair e espera a sua confirmação.
- **Arquivos pessoais vão para a Lixeira.** Só caches, temporários, logs e pastas de build são removidos de vez.
- **Locais protegidos.** Pastas do sistema, seus Documentos, Mesa e Imagens, pastas `.git` e o interior de apps e bibliotecas de fotos nunca podem ser apagados.
- **Conferido de novo no último instante.** Cada item é olhado de novo logo antes da remoção. Se mudou desde a prévia, é pulado.
- **Sem downloads de surpresa.** A varredura nunca faz o macOS baixar arquivos que estão só no iCloud.
- **Privilégio de administrador só para uma operação.** O app nunca roda como administrador e não deixa nenhum serviço em segundo plano. O próprio macOS pede o Touch ID ou a senha, e o app nunca a vê.
- **Sem rede, sem telemetria.** O app nunca se conecta à internet.

Detalhes no [guia de segurança](docs/guide/safety.pt-BR.md).

## Como usar

**No app**, pressione `1` a `7` para trocar de tela. Clique no ícone de controles, no canto superior direito, para mudar a paleta e a quantidade de animação. Toda tela que remove alguma coisa funciona do mesmo jeito: primeiro procura, mostra o que encontrou, deixa você escolher e pede confirmação.

**No terminal**, o `scan` mostra um resumo e os maiores itens de uma pasta. Ele só lê:

```text
Pasta              /Users/voce/Projetos
Itens              1.386.543
Ocupado em disco   24,2 GB
Tamanho lógico     20,6 GB
Somente na nuvem   0 B
Pastas sem acesso  0
Tempo              9,3 s

Maiores itens
     12,5 GB  portal-web/
     11,7 GB  app-mobile/
```

O [guia de uso](docs/guide/usage.pt-BR.md) explica cada tela, cada botão e cada linha dessa saída.

## Tamanho e uso do processador

Medido em um Apple M2 Pro:

- O app instalado ocupa cerca de 10 MB.
- Parado, ele usa mais ou menos 3% de um núcleo do processador enquanto a janela não está na frente, e de 10 a 15% enquanto está na frente com as animações completas. Mudar "Animações" para "Reduzida" diminui esse uso.

## Como é construído

O MacCleaner segue a Clean Architecture. Cada camada é um crate próprio, então o compilador recusa uma dependência que aponte para o lado errado.

```mermaid
flowchart LR
    apps["apps: desktop, cli, privileged-helper"] --> ui["maccleaner-ui"]
    apps --> infra["maccleaner-infrastructure"]
    ui --> application["maccleaner-application"]
    infra --> application
    application --> domain["maccleaner-domain"]
```

| Crate | Papel |
|---|---|
| `maccleaner-domain` | Regras sem acesso a disco: árvore de arquivos, política de exclusão, regras de limpeza, duplicados, semelhança de fotos, desenho do mapa, tamanhos |
| `maccleaner-application` | Casos de uso e as portas de que eles precisam |
| `maccleaner-infrastructure` | Adaptadores do macOS: disco, Lixeira, histórico, hash de conteúdo, miniaturas de imagens, leituras do sistema, permissões, elevação de privilégio |
| `maccleaner-ui` | A interface em egui: tema, componentes, as sete telas |
| `apps/desktop`, `apps/cli`, `apps/privileged-helper` | Binários que ligam tudo |
| `xtask` | Tarefas do projeto: `check`, `bundle`, `install` |

Um comando roda formatação, lints, testes e a auditoria de dependências:

```bash
cargo xtask check
```

A interface segue uma referência de design guardada em [`design/prototype/index.html`](design/prototype/index.html).

## Documentação

| Guia | English | Português |
|---|---|---|
| Primeiros passos: instalar, compilar, abrir | [getting-started.md](docs/guide/getting-started.md) | [getting-started.pt-BR.md](docs/guide/getting-started.pt-BR.md) |
| Uso: cada tela e a linha de comando | [usage.md](docs/guide/usage.md) | [usage.pt-BR.md](docs/guide/usage.pt-BR.md) |
| Segurança: o que pode e o que não pode ser apagado | [safety.md](docs/guide/safety.md) | [safety.pt-BR.md](docs/guide/safety.pt-BR.md) |
| Perguntas e solução de problemas | [faq.md](docs/guide/faq.md) | [faq.pt-BR.md](docs/guide/faq.pt-BR.md) |

Para quem contribui (em inglês): [arquitetura](docs/architecture.md), [convenções](docs/conventions.md), [política de segurança](docs/security.md) e [registros de decisão](docs/adr/).

## O que falta

**Verificar.** Tudo o que está em [Ainda não verificado](#ainda-não-verificado): uma limpeza de sistema em um Mac de verdade, do diálogo de autenticação até o resultado; uma varredura sobre um arquivo que esteja realmente só na nuvem; uma abertura com "Reduzir movimento" ligado; a remoção de dados pessoais reais; e o uso em outras versões do macOS e em Macs com Intel. Relatos são bem-vindos, inclusive os que dizem "funcionou".

**Conviver por enquanto.** Os [limites conhecidos](#limites-conhecidos) acima.

**Ideias, não compromissos.**

- Um `.app` assinado e notarizado para baixar, tornando opcional compilar a partir do código.
- Interface em inglês. Todos os textos já ficam em um único arquivo.
- Lembrar a paleta e o nível de animação entre uma abertura e outra.
- Limpar mais áreas do sistema, como os snapshots locais do Time Machine e logs antigos em `/private/var/log`.
- Velocidade das ventoinhas no monitor.

## Como contribuir

Contribuições são bem-vindas, em português ou inglês. Comece pelo [CONTRIBUTING.md](CONTRIBUTING.md). Para relatar uma vulnerabilidade, siga o [SECURITY.md](SECURITY.md) em vez de abrir uma issue pública.

## Licença

[MIT](LICENSE). As fontes embarcadas, JetBrains Mono e Rajdhani, são licenciadas sob a SIL Open Font License; os textos das licenças estão em [`assets/fonts/`](assets/fonts/).
