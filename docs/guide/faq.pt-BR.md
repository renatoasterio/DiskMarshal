# Perguntas e solução de problemas

[English](faq.md) · Português

## Perguntas

### O MacCleaner apaga alguma coisa hoje?

Não. O app mostra a moldura enquanto as telas são construídas, e o `maccleaner-cli` só lê. Veja o [estado do projeto](../../README.pt-BR.md#estado-do-projeto).

### É seguro rodar?

Sim. Hoje ele só lê nomes e tamanhos de arquivos. Quando a limpeza chegar, vai seguir as regras do [guia de segurança](safety.pt-BR.md): prévia antes, arquivos pessoais na Lixeira e uma lista de lugares que nunca podem ser apagados.

### Ele envia meus dados para algum lugar?

Não. O MacCleaner nunca se conecta à internet. Não há telemetria, conta nem busca por atualizações.

### A varredura vai baixar meus arquivos do iCloud?

Não. O MacCleaner avisa o macOS para não baixar arquivos que estão só na nuvem durante a varredura. Eles são medidos e identificados, não baixados.

### Por que não tem um app pronto para baixar?

O projeto é novo, e um app para download precisa ser assinado com um certificado de desenvolvedor da Apple. Um `.app` assinado está nos [próximos passos](../../README.pt-BR.md#próximos-passos). Por enquanto você compila a partir do código, o que o [guia de primeiros passos](getting-started.pt-BR.md) mostra em detalhes.

### Em quais Macs e versões do macOS funciona?

Ele é desenvolvido e testado no macOS 27 em Apple Silicon. Outras versões e Macs com Intel devem funcionar, mas não foram testados. Se você testar em um deles, uma issue contando como foi é muito bem-vinda.

### Funciona no Windows ou no Linux?

Não. O MacCleaner é construído sobre recursos do macOS, como os arquivos do iCloud que ficam só na nuvem, a Lixeira e os sensores do sistema.

### A interface vai ter outros idiomas?

Hoje ela é em português. Todos os textos da interface ficam em um único arquivo, então adicionar inglês é uma tarefa bem delimitada. Contribuições são bem-vindas.

### É seguro mesmo apagar `node_modules`, `target` e pastas parecidas?

Sim, desde que o projeto ainda tenha o arquivo que o descreve (`package.json`, `Cargo.toml` e assim por diante). Essas pastas são geradas: suas ferramentas as recriam na próxima instalação ou compilação. O MacCleaner só vai oferecer uma pasta quando encontrar esse arquivo ao lado dela.

### Qual a diferença entre "Ocupado em disco" e "Tamanho lógico"?

"Tamanho lógico" é o tamanho do conteúdo. "Ocupado em disco" é o espaço que ele realmente toma, que é o que você recupera ao apagar. Eles diferem porque o disco guarda arquivos em blocos e pode compactá-los.

### No que ele é diferente de outros limpadores?

O código é aberto, então qualquer pessoa pode conferir o que ele faz. Ele nunca apaga sem prévia, nunca mexe no interior da sua biblioteca do Fotos e nunca envia nada pela rede.

## Solução de problemas

Procure na coluna da esquerda a mensagem que você está vendo.

| O que aparece | Por quê | Como resolver |
|---|---|---|
| `command not found: cargo` | O Terminal não sabe onde o Rust está. | Rode os dois comandos do [passo 3](getting-started.pt-BR.md#passo-3-instale-o-rust) que começam com `echo` e `source` e tente de novo. |
| `command not found: brew` | O Homebrew não está instalado, ou os "Next steps" dele foram pulados. | Siga o [passo 2](getting-started.pt-BR.md#passo-2-instale-o-homebrew), incluindo os comandos que o Homebrew mostra no final. |
| `xcrun: error: invalid active developer path` | Faltam as ferramentas de linha de comando do Xcode. Acontece muito depois de atualizar o macOS. | Rode `xcode-select --install`. |
| `error: could not find Cargo.toml` | O comando rodou fora da pasta do projeto. | Rode `cd ~/MacCleaner` antes. |
| `error: no such command: xtask` | Mesma causa: o comando rodou fora da pasta do projeto. | Rode `cd ~/MacCleaner` antes. |
| `error: no such command: deny` durante o `cargo xtask check` | O auditor de dependências não está instalado. | Rode `brew install cargo-deny`. |
| A compilação para com `No space left on device` | O disco está cheio. A compilação precisa de cerca de 1 GB. | Libere espaço e rode o comando de novo. |
| "Pastas sem acesso" é maior que zero | O macOS não deixou o Terminal ler algumas pastas. | Dê ao Terminal o [Acesso Total ao Disco](usage.pt-BR.md#quando-pastas-sem-acesso-não-é-zero). |
| O macOS pergunta se o Terminal pode acessar Documentos, Mesa ou Downloads | Proteção normal do macOS para essas pastas. | Clique em **Permitir** para incluí-las na varredura, ou em **Não Permitir** para deixá-las de fora. |
| `erro: não foi possível impedir downloads do iCloud` | O macOS recusou a configuração que impede downloads do iCloud, então o MacCleaner parou de propósito. | Por favor, [abra uma issue](https://github.com/renatoasterio/MacCleaner/issues) informando a versão do seu macOS. |
| A janela fica lenta ou as ventoinhas aceleram | As animações redesenham a janela o tempo todo. | Abra o painel de aparência e mude "Animações" para "Reduzida" ou "Desligada". |
| A pasta do projeto ocupa muito espaço | A saída da compilação fica na pasta `target`. | Rode `cargo clean` dentro da pasta do projeto. A próxima compilação recria tudo. |

## Ainda com problema?

Abra uma issue em [github.com/renatoasterio/MacCleaner/issues](https://github.com/renatoasterio/MacCleaner/issues). Inclua:

- a versão do seu macOS (menu Apple, **Sobre Este Mac**);
- o comando que você rodou;
- a mensagem completa que o Terminal mostrou.

Issues em português ou em inglês são bem-vindas.
