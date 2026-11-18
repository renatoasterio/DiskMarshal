# Perguntas e solução de problemas

[English](faq.md) · Português

## Perguntas

### O DiskMarshal apaga alguma coisa?

Sim, mas só o que você seleciona, e só depois de mostrar uma prévia e de você confirmar. Abrir o app, procurar e analisar nunca removem nada. Arquivos duplicados, fotos parecidas e os Archives do Xcode vão para a Lixeira; caches, temporários, logs e pastas de build são removidos de vez. A ferramenta de linha de comando, `diskmarshal-cli`, só lê.

### É seguro rodar?

Ele segue as regras do [guia de segurança](safety.pt-BR.md): prévia antes, arquivos pessoais na Lixeira, uma lista de lugares que nunca podem ser apagados, uma segunda conferência de cada item logo antes da remoção e um histórico por escrito. Essas regras são cobertas por testes automáticos.

Saiba que o projeto é novo e não tem uma versão estável. Algumas coisas ainda não foram verificadas na situação real; a próxima pergunta lista quais. Comece com uma limpeza pequena e olhe o resultado.

### O que ainda não foi testado?

- **Partes da limpeza de sistema.** A limpeza de `/Library/Logs` com a digital e a desinstalação de dois apps de verdade pelo diálogo de senha foram confirmadas à mão em um Mac. Uma limpeza real de `/Library/Caches`, a limpeza de sistema pelo diálogo de senha e o diálogo de senha que aparece depois de uma digital cancelada, não. O auxiliar que faz isso é coberto por testes em pastas temporárias e pelo seu modo de simulação.
- **Um arquivo que está realmente só no iCloud.** A proteção contra downloads existe, mas não foi experimentada com um arquivo que esteja de fato só na nuvem.
- **Abrir com movimento reduzido,** com a opção "Reduzir movimento" dos Ajustes do Sistema ligada.
- **Apagar dados pessoais reais pela interface.** Isso foi exercitado apenas em pastas de teste.
- **Outras versões do macOS e Macs com Intel.**

### Ele envia meus dados para algum lugar?

Não. O DiskMarshal nunca se conecta à internet. Não há telemetria, conta nem busca por atualizações.

### A varredura vai baixar meus arquivos do iCloud?

Não. O DiskMarshal avisa o macOS para não baixar arquivos que estão só na nuvem durante a varredura. Eles são medidos e identificados, não baixados. As buscas de duplicados e de fotos parecidas os pulam por completo.

### Por que não tem um app pronto para baixar?

Um app para download precisa ser assinado com um certificado de desenvolvedor da Apple e conferido pela Apple. O projeto não oferece isso hoje; é uma ideia, não uma promessa. Por enquanto você compila a partir do código, uma vez, com `cargo xtask install`. O resultado é um app comum na pasta Aplicativos, que abre pelo Launchpad ou pelo Spotlight. O [guia de primeiros passos](getting-started.pt-BR.md) mostra tudo em detalhes.

### Preciso do Terminal toda vez?

Não. Você precisa dele uma vez, para compilar e instalar. Depois disso o DiskMarshal abre como qualquer outro app. O Terminal só volta a ser necessário para atualizar para uma versão mais nova.

### O DiskMarshal precisa do Acesso Total ao Disco?

Não, ele funciona sem. Mas o macOS então esconde do app algumas pastas, como os dados do Mail e do Safari e a Lixeira, e elas não aparecem nos resultados. As telas que leem as suas pastas mostram um aviso de uma linha com o link "Abrir ajustes de Acesso Total ao Disco". O passo a passo está no [guia de primeiros passos](getting-started.pt-BR.md#passo-7-dê-o-acesso-total-ao-disco-opcional).

### Por que o macOS esqueceu o Acesso Total ao Disco que eu dei?

Porque o app foi recompilado. O app que você compila leva uma assinatura local (ad hoc) que muda a cada compilação, então depois de cada `cargo xtask install` o macOS enxerga um app novo. Dê a permissão de novo, ou assine com uma identidade sua, definindo a variável de ambiente `DISKMARSHAL_SIGNING_IDENTITY` antes de rodar `cargo xtask install`.

### Como a limpeza das áreas de sistema pede a minha senha?

Ela não pede; quem pede é o macOS. Quando você confirma uma limpeza que inclui o grupo "Sistema", o macOS mostra o diálogo de autenticação dele e recebe a sua senha de administrador, ou a sua digital se você a ativou. O DiskMarshal nunca vê nenhuma das duas. O macOS então roda, uma única vez, um pequeno auxiliar que fica dentro do app, e o auxiliar encerra quando termina. O [guia de segurança](safety.pt-BR.md#áreas-de-administrador) explica os detalhes.

### O diálogo vai oferecer o Touch ID?

Só depois que você ativar. O diálogo de senha do macOS não aceita a digital. Depois que você ativa o Touch ID para o `sudo`, com uma linha no Terminal mostrada no [guia de uso](usage.pt-BR.md#usar-a-digital-em-vez-da-senha), o DiskMarshal pede primeiro a digital e mostra o diálogo de senha quando ela não passa. O caminho da digital não foi verificado pelos testes automáticos do projeto: precisa de um dedo no sensor.

### O DiskMarshal roda como administrador ou deixa algo rodando em segundo plano?

Não para as duas coisas. O app sempre roda como o seu próprio usuário. O privilégio de administrador existe só dentro do pequeno auxiliar, para uma limpeza, depois que o macOS autentica você. Nenhum serviço é instalado e nada continua rodando depois.

### Por que o disco liberou menos espaço do que o app estimou?

Alguns motivos possíveis:

- O que foi para a Lixeira continua ocupando espaço até você esvaziá-la.
- O formato de disco do Mac (APFS) pode guardar duas cópias de um arquivo como uma só, e pode manter versões antigas em snapshots. Remover um arquivo cujo conteúdo ainda é usado em outro lugar libera menos que o tamanho dele.
- Alguns itens não puderam ser removidos (o macOS os protegeu, ou eles mudaram depois da prévia). O diálogo de resultado conta esses como "itens deixados como estavam".

### Por que as fotos do meu app Fotos não são encontradas?

A tela "Fotos" compara só imagens soltas em pastas. A biblioteca do app Fotos é um pacote lacrado, e o DiskMarshal nunca a lê nem a altera. Para remover duplicadas dentro dessa biblioteca, use o próprio app Fotos.

### Por que o monitor não mostra a velocidade das ventoinhas?

O monitor não lê a velocidade das ventoinhas; é um limite conhecido. Ele mostra processador, memória, swap, disco, temperaturas e os programas mais ativos.

### Quanto de disco e de processador o app usa?

Medido em um Apple M2 Pro: o app instalado ocupa cerca de 10 MB. Parado, ele usa mais ou menos 3% de um núcleo do processador enquanto a janela não está na frente, e de 10 a 15% enquanto está na frente com as animações completas. Mudar "Animações" para "Reduzida", no painel de aparência, diminui esse uso.

### Em quais Macs e versões do macOS funciona?

Ele é desenvolvido e testado no macOS 27 em Apple Silicon. Outras versões e Macs com Intel devem funcionar, mas não foram testados. Se você testar em um deles, uma issue contando como foi é muito bem-vinda.

### Funciona no Windows ou no Linux?

Não. O DiskMarshal é construído sobre recursos do macOS, como os arquivos do iCloud que ficam só na nuvem, a Lixeira e os sensores do sistema.

### A interface vai ter outros idiomas?

Hoje ela é em português. Todos os textos da interface ficam em um único arquivo, então adicionar inglês é uma tarefa bem delimitada. Contribuições são bem-vindas.

### É seguro mesmo apagar `node_modules`, `target` e pastas parecidas?

Sim, desde que o projeto ainda tenha o arquivo que o descreve (`package.json`, `Cargo.toml` e assim por diante). Essas pastas são recriadas pelas suas ferramentas na próxima instalação ou compilação. O DiskMarshal só oferece uma pasta quando encontra esse arquivo ao lado dela e o git não tem nenhum arquivo versionado dentro da pasta. A remoção é permanente: a pasta não vai para a Lixeira.

### Qual a diferença entre "Ocupado em disco" e "Tamanho lógico"?

"Tamanho lógico" é o tamanho do conteúdo. "Ocupado em disco" é o espaço que ele realmente toma, que é o que você recupera ao apagar. Eles diferem porque o disco guarda arquivos em blocos e pode compactá-los.

### No que ele é diferente de outros limpadores?

O código é aberto, então qualquer pessoa pode conferir o que ele faz. Ele nunca apaga sem prévia, nunca mexe no interior da sua biblioteca do Fotos e nunca envia nada pela rede.

### Por que o desinstalador deixa alguns arquivos de um app?

Ele remove o que leva o identificador do app ou o nome dele, nas pastas da Biblioteca onde os apps guardam suas coisas. Um documento que o app salvou nas suas pastas é seu e fica. Uma pasta com o nome do fabricante do app também fica, porque outros apps do mesmo fabricante podem usá-la. Os itens com "Revisar" também são encontrados, mas esperam você selecionar.

### Por que não consigo selecionar um app, ou alguns itens dele?

Há dois motivos. O app está aberto: ele aparece marcado com "aberto" na lista, então feche o app e clique nele de novo. Ou o item precisa de administrador e você está rodando a partir do código-fonte: o grupo "Sistema" mostra então "só no app instalado · cargo xtask install".

### Por que limpar o Docker não liberou espaço no meu Mac na hora?

O espaço é liberado dentro do Docker. O Docker Desktop guarda tudo em um único arquivo grande no Mac, e esse arquivo pode não diminuir na hora. O diálogo de resultado mostra o que o Docker liberou dentro dele. Se e quando o Docker Desktop devolve esse espaço ao macOS não foi conferido por este projeto.

### Como desinstalo?

Mova o `/Applications/DiskMarshal.app` para a Lixeira. O histórico de remoções fica em `~/Library/Application Support/DiskMarshal/`; apague essa pasta se não quiser guardá-lo. Se não precisar mais do código, mova também a pasta `DiskMarshal` do projeto para a Lixeira.

## Solução de problemas

Procure na coluna da esquerda a mensagem que você está vendo.

### Durante a instalação

| O que aparece | Por quê | Como resolver |
|---|---|---|
| `command not found: cargo` | O Terminal não sabe onde o Rust está. | Rode os dois comandos do [passo 3](getting-started.pt-BR.md#passo-3-instale-o-rust) que começam com `echo` e `source` e tente de novo. |
| `command not found: brew` | O Homebrew não está instalado, ou os "Next steps" dele foram pulados. | Siga o [passo 2](getting-started.pt-BR.md#passo-2-instale-o-homebrew), incluindo os comandos que o Homebrew mostra no final. |
| `xcrun: error: invalid active developer path` | Faltam as ferramentas de linha de comando do Xcode. Acontece muito depois de atualizar o macOS. | Rode `xcode-select --install`. |
| `error: could not find Cargo.toml` | O comando rodou fora da pasta do projeto. | Rode `cd ~/DiskMarshal` antes. |
| `error: no such command: xtask` | Mesma causa: o comando rodou fora da pasta do projeto. | Rode `cd ~/DiskMarshal` antes. |
| `error: no such command: deny` durante o `cargo xtask check` | O auditor de dependências não está instalado. | Rode `brew install cargo-deny`. |
| A compilação para com `No space left on device` | O disco está cheio. A compilação precisa de cerca de 1 GB. | Libere espaço e rode o comando de novo. |
| `xtask: could not replace the installed app` ou `xtask: could not copy the app to the destination folder` | O mais provável é que a sua conta não possa gravar em `/Applications`. | Instale na sua própria pasta Aplicativos: `cargo xtask install ~/Applications`. |
| `xtask: could not sign the app` | O mais provável é que o nome em `DISKMARSHAL_SIGNING_IDENTITY` não corresponda a uma identidade de assinatura deste Mac. | Corrija o nome, ou remova a variável para usar a assinatura local. |
| A pasta do projeto ocupa muito espaço | A saída da compilação fica na pasta `target`. | Rode `cargo clean` dentro da pasta do projeto. O app instalado continua funcionando, e a próxima compilação recria a pasta. |

### No app

| O que aparece | Por quê | Como resolver |
|---|---|---|
| "Sem Acesso Total ao Disco, o macOS esconde parte da sua pasta pessoal das análises." | O app não tem o Acesso Total ao Disco, então o macOS esconde algumas pastas dele. | Opcional. Clique em "Abrir ajustes de Acesso Total ao Disco" e siga o [passo 7](getting-started.pt-BR.md#passo-7-dê-o-acesso-total-ao-disco-opcional). |
| Esse aviso voltou depois de uma atualização | O macOS esquece a permissão quando o app é recompilado com a assinatura local. | Dê a permissão de novo. Veja [a pergunta acima](#por-que-o-macos-esqueceu-o-acesso-total-ao-disco-que-eu-dei). |
| A linha "Lixeira" mostra "vazio" mesmo com itens na Lixeira | O macOS só mostra a Lixeira a apps com Acesso Total ao Disco. | Dê o Acesso Total ao Disco, ou esvazie a Lixeira pelo Finder. |
| O grupo "Sistema" mostra "só no app instalado · cargo xtask install" e não pode ser selecionado | O app está rodando direto do código, sem o auxiliar que limpa as áreas de sistema. | Instale o app com `cargo xtask install` e abra o app instalado. |
| "As áreas de sistema não foram limpas: a autenticação foi cancelada." | O diálogo de autenticação do macOS foi fechado. | Nada foi removido das áreas de sistema. Rode a limpeza de novo e autentique, se quiser limpá-las. |
| "As áreas de sistema não foram limpas: esse recurso só existe no app instalado." | O auxiliar não foi encontrado, foi alterado, ou a assinatura do app deixou de ser válida. | Rode `cargo xtask install` de novo e abra o app instalado. |
| "As áreas de sistema não foram limpas:" seguido de outro motivo | A limpeza começou e não conseguiu terminar. | Nada é forçado. Por favor, [abra uma issue](https://github.com/renatoasterio/DiskMarshal/issues) com a mensagem. |
| "itens deixados como estavam: mudaram, sumiram ou o macOS não permitiu" | Alguns itens mudaram depois da prévia, já não existiam ou são protegidos pelo macOS. Foram pulados de propósito. | Nada a resolver. A tela procura de novo depois que você fecha o diálogo. |
| "Selecione ao menos um item." | O botão de ação foi clicado sem nada selecionado. | Selecione alguma coisa antes. |
| "Ao menos uma cópia de cada arquivo precisa ficar." ou "Ao menos uma foto de cada grupo precisa ficar." | Você tentou marcar todos os arquivos de um grupo para remoção. | Mantenha um. Para remover todos, use o Finder. |
| "Análise interrompida" | A análise da limpeza não conseguiu terminar. | Clique em "Tentar de novo". |
| "Aguardando leituras" não sai do monitor | O macOS não informou as leituras do sistema. | Espere alguns segundos. Se continuar, por favor [abra uma issue](https://github.com/renatoasterio/DiskMarshal/issues) informando a versão do seu macOS. |
| O macOS pergunta se o DiskMarshal pode acessar Documentos, Mesa ou Downloads | Proteção normal do macOS para essas pastas. | Clique em **Permitir** para incluí-las, ou em **Não Permitir** para deixá-las de fora. |
| A janela fica lenta, ou o Mac esquenta | As animações redesenham a janela o tempo todo enquanto ela está na frente. | Abra o painel de aparência e mude "Animações" para "Reduzida" ou "Desligada". |

### Na ferramenta de linha de comando

| O que aparece | Por quê | Como resolver |
|---|---|---|
| "Pastas sem acesso" é maior que zero | O macOS não deixou o Terminal ler algumas pastas. | Dê ao Terminal o [Acesso Total ao Disco](usage.pt-BR.md#quando-pastas-sem-acesso-não-é-zero). |
| O macOS pergunta se o Terminal pode acessar Documentos, Mesa ou Downloads | Proteção normal do macOS para essas pastas. | Clique em **Permitir** para incluí-las na varredura, ou em **Não Permitir** para deixá-las de fora. |
| `erro: não foi possível impedir downloads do iCloud` | O macOS recusou a configuração que impede downloads do iCloud, então o DiskMarshal parou de propósito. | Por favor, [abra uma issue](https://github.com/renatoasterio/DiskMarshal/issues) informando a versão do seu macOS. |

## Ainda com problema?

Abra uma issue em [github.com/renatoasterio/DiskMarshal/issues](https://github.com/renatoasterio/DiskMarshal/issues). Inclua:

- a versão do seu macOS (menu Apple, **Sobre Este Mac**);
- o que você estava fazendo: a tela e o botão, ou o comando que rodou;
- a mensagem completa que o app ou o Terminal mostrou.

Issues em português ou em inglês são bem-vindas.
