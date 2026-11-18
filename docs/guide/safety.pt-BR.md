# Segurança

[English](safety.md) · Português

Um limpador apaga arquivos, então um erro pode custar dados. Esta página explica, em linguagem simples, as regras que o MacCleaner segue para isso não acontecer, e diz com honestidade o que ainda não foi testado.

> **Como as coisas estão hoje.** O app apaga de verdade o que você seleciona. Todas as regras desta página são impostas no código e cobertas por testes automáticos. Esses testes rodam em pastas temporárias criadas para isso; a remoção de dados pessoais reais pela interface só foi experimentada em pastas de teste. Comece pequeno: limpe uma coisa, olhe o resultado, depois siga. A seção [O que não foi testado](#o-que-não-foi-testado) lista o resto.

## As cinco regras

### 1. Você sempre vê uma prévia antes

Antes de remover qualquer coisa, o MacCleaner lista exatamente o que vai sair e quanto espaço isso libera. Procurar e analisar nunca removem nada. Depois, um diálogo chamado "Autorização requerida" repete quantos itens e quanto espaço, e espera. Nada acontece até você clicar no botão que confirma. "Cancelar" fecha o diálogo sem remover nada.

![A tela de limpeza listando cada regra, onde ela procura e quanto encontrou, antes de qualquer remoção](../assets/screens/cleanup.png)

*Captura do app com dados de demonstração.*

### 2. Arquivos pessoais vão para a Lixeira

| O que está sendo removido | Tela | Para onde vai | Dá para recuperar? |
|---|---|---|---|
| Arquivos duplicados | "Duplicados" | A Lixeira | Sim. Abra a Lixeira, clique com o botão direito no item e escolha **Colocar de Volta**. |
| Fotos parecidas | "Fotos" | A Lixeira | Sim, do mesmo jeito. |
| "Xcode Archives" | "Limpeza" | A Lixeira | Sim, do mesmo jeito. |
| Caches, temporários, logs, resultado de compilações do Xcode, caches de gerenciadores de pacotes | "Limpeza" | Removidos na hora | Não, mas os apps e as ferramentas recriam quando precisam. |
| Caches e logs do sistema (`/Library/Caches`, `/Library/Logs`) | "Limpeza", grupo "Sistema" | Removidos na hora | Não, mas o sistema e os apps recriam quando precisam. |
| Pastas de build e dependências (`node_modules`, `target`, `bin`/`obj`) | "Desenvolvedor" | Removidas na hora | Não, mas suas ferramentas recriam na próxima instalação ou compilação. |
| Um app que você mesmo instalou, e o que ele deixou na sua Biblioteca | "Aplicativos" | A Lixeira | Sim, do mesmo jeito. |
| Um app que a App Store ou um instalador colocou ali, e o que ele deixou em `/Library` | "Aplicativos", grupo "Sistema" | Removidos na hora | Não. O app pode ser instalado de novo. O que ele guardava em `/Library` não volta. |
| Cache de build, imagens, containers parados e volumes do Docker | "Docker" | Removidos na hora, pelo próprio Docker | O cache de build e as imagens voltam com um novo build ou download. **O conteúdo de um container ou de um volume não volta.** |
| O que já está na Lixeira | "Limpeza", regra "Lixeira" | Removido na hora | **Não.** Essa regra esvazia a Lixeira. |

Só é removido de vez o que pode ser recriado, com três exceções, e cada uma depende de uma escolha sua:

- **A regra "Lixeira"** esvazia a Lixeira, e isso não tem volta. Ela tem a etiqueta "Revisar" e nunca vem selecionada.
- **Um container ou um volume do Docker** guarda conteúdo que não existe em nenhum outro lugar. Os dois têm a etiqueta "Revisar" e nunca vêm selecionados.
- **A parte de administrador de uma desinstalação.** Foi você quem escolheu desinstalar aquele app, e o que ele guardava em `/Library` sai junto, de vez.

O espaço do que foi para a Lixeira só é liberado quando você a esvazia.

### 3. Alguns lugares nunca podem ser apagados

O MacCleaner se recusa a apagar estes itens, mesmo que um defeito ou uma regra errada peça:

- **Qualquer coisa dentro das pastas do próprio sistema:** `/System`, `/usr`, `/bin` e `/sbin`.
- **As pastas compartilhadas em si:** `/Applications`, `/Library`, `/Library/Caches`, `/Library/Logs`, `/Users`, `/Volumes`, `/private`, `/tmp` e algumas outras. O conteúdo de `/Library/Caches` e de `/Library/Logs` pode ser limpo pelo grupo "Sistema"; as pastas ficam.
- **As suas pastas pessoais em si:** a pasta pessoal, Mesa, Documentos, Downloads, Imagens, Filmes, Música, Público, Sites, a sua própria pasta Aplicativos e a pasta da Lixeira.
- **As pastas da Biblioteca em si:** `~/Library`, `~/Library/Caches`, `~/Library/Logs`, `~/Library/Application Support`, `~/Library/Containers`, `~/Library/Group Containers`, `~/Library/Developer`, `~/Library/Preferences` e `~/Library/Mobile Documents` (onde fica o iCloud Drive). O conteúdo pode ser limpo; as pastas ficam.
- **Qualquer coisa dentro de uma pasta `.git`,** que guarda o histórico de um projeto de código, e a própria pasta `.git`.
- **Qualquer coisa dentro de um pacote:** um app (`.app`), uma biblioteca do Fotos (`.photoslibrary`), uma biblioteca do Música ou do TV, uma biblioteca do iMovie ou do Final Cut, um projeto do Logic, uma imagem de disco em pacote (`.sparsebundle`) e parecidos. Apagar pedaços deles os corrompe, então o MacCleaner trata cada um como um item lacrado.
- **Qualquer coisa fora das pastas em que a busca foi feita,** e a própria pasta da busca.

Trocar maiúsculas por minúsculas não contorna isso: `/system` é tratado como `/System`. Um caminho escrito com `..` é recusado de cara.

### 4. Cada item é conferido de novo logo antes da remoção

Passa um tempo entre a prévia e o seu clique. Por isso, imediatamente antes de remover um item, o MacCleaner olha para ele de novo. Se ele mudou, foi movido ou substituído desde que apareceu na prévia, é pulado e informado, não removido. O diálogo de resultado conta esses como "itens deixados como estavam".

Ele também nunca segue atalhos (links simbólicos): um link dentro de uma pasta de cache é removido como link, e aquilo para onde ele aponta fica intocado. E nunca alcança outro disco: uma pasta que tenha outro disco montado dentro dela é recusada inteira.

### 5. Tudo o que é removido fica registrado

Cada remoção é gravada em um arquivo de histórico que só a sua conta de usuário consegue ler:

```text
~/Library/Application Support/MacCleaner/history.jsonl
```

Cada linha registra quando aconteceu, o que foi removido, quanto espaço ocupava e se foi apagado ou enviado para a Lixeira. O "Painel" mostra as entradas mais recentes em "Últimas remoções". O que a limpeza de sistema remove também fica registrado ali.

Se uma remoção aconteceu mas não pôde ser gravada no histórico, o diálogo de resultado avisa.

## Um único caminho de exclusão para tudo

Toda tela que remove alguma coisa usa o mesmo trecho de código para isso, e esse código aplica as cinco regras acima. Não existe um segundo jeito, mais rápido, de apagar escondido em algum canto. O auxiliar que limpa as áreas de sistema e remove os itens de sistema de um app desinstalado usa esse mesmo código.

Uma coisa não tem como passar por ele: o que o Docker guarda. Isso não é feito de arquivos que o app alcance, então é o próprio Docker quem remove, um item por vez, e ele recusa o que estiver em uso. O MacCleaner nunca força o Docker a remover nada.

## O que cada tela preserva para você

- **"Duplicados":** ao menos uma cópia de cada arquivo sempre fica. Por padrão, a cópia mais antiga é mantida.
- **"Fotos":** ao menos uma foto de cada grupo sempre fica. Por padrão, o maior arquivo é mantido. A biblioteca do app Fotos nunca é lida nem alterada.
- **"Desenvolvedor":** uma pasta só é oferecida quando o arquivo que identifica o projeto está ao lado dela e o git não tem nenhum arquivo versionado dentro dela. Nenhum código-fonte é removido.
- **"Limpeza":** uma regra remove o conteúdo das pastas dela, nunca as pastas. As regras marcadas com "Revisar" e o grupo "Sistema" inteiro nunca vêm selecionados.
- **"Aplicativos":** um app que está aberto é recusado. Só são encontrados os itens que levam o nome do app, e só os que batem exatamente vêm selecionados. Quando há outra cópia do app instalada, os dados ficam.
- **"Docker":** o que um container usa nunca entra na lista, e o Docker recusa o que tiver passado a ser usado nesse meio-tempo. Imagens com nome, containers e volumes nunca vêm selecionados.

## Arquivos do iCloud não são baixados

Arquivos guardados só no iCloud não ocupam espaço no seu Mac. Normalmente, qualquer programa que lê um arquivo desses faz o macOS baixá-lo, o que encheria justamente o disco que você quer liberar.

O MacCleaner avisa o macOS, antes de ler qualquer coisa, que ele não deve baixar arquivos por causa do app. Itens que estão só na nuvem são medidos e identificados, nunca baixados. As buscas de duplicados e de fotos parecidas os deixam de fora, porque compará-los exigiria lê-los. Se o macOS recusar essa configuração, o MacCleaner para em vez de varrer.

Isso foi testado com arquivos guardados no Mac. Não foi testado com um arquivo que esteja realmente só na nuvem.

## Sem rede, sem telemetria

O MacCleaner não se conecta à internet. Não envia dados de uso, não procura atualizações e não tem conta. O que ele descobre sobre o seu disco fica no seu Mac.

## Áreas de administrador

Dois lugares exigem privilégio de administrador: `/Library/Caches` ("Caches do sistema") e `/Library/Logs` ("Logs do sistema"). Eles ficam no grupo "Sistema" da tela de limpeza e nunca vêm selecionados.

Como funciona quando você os seleciona e confirma:

1. **O MacCleaner pede ao macOS que autentique você.** O macOS mostra o diálogo dele, pedindo o Touch ID ou a senha de administrador. A senha vai para o macOS. O MacCleaner nunca a vê e nunca a guarda.
2. **O macOS roda um pequeno auxiliar, uma única vez.** O auxiliar é um programinha separado que fica dentro do app. Ele limpa as áreas que você selecionou, informa o que fez e encerra.
3. **O MacCleaner grava no seu histórico o que foi removido,** como em qualquer outra remoção.

O que mantém isso restrito:

- **O app em si nunca roda como administrador.** Só o auxiliar roda, e só para aquela operação.
- **Nada fica em segundo plano.** Nenhum serviço é instalado. Toda limpeza de sistema pede autenticação de novo.
- **O auxiliar aceita só os nomes de regras de limpeza conhecidas,** como `system-caches`. Ele recusa qualquer coisa com cara de caminho ou de opção. Quais pastas uma regra cobre está escrito dentro do próprio auxiliar, então ninguém consegue apontá-lo para outra pasta.
- **O auxiliar só aceita regras feitas para administrador,** olha as pastas de novo por conta própria e remove pelas mesmas regras de exclusão do app (as regras 3 e 4 acima).
- **O app confere o auxiliar antes de usá-lo:** ele precisa ser um arquivo comum (não um atalho) que só o dono pode alterar e, no app instalado, o macOS precisa confirmar que o app continua exatamente como foi assinado.

O que você precisa saber:

- **Esse caminho não foi verificado de ponta a ponta.** O auxiliar é coberto por testes automáticos em pastas temporárias e pelo seu modo de simulação, que confere tudo e não remove nada. O diálogo de autenticação do macOS e uma limpeza real de `/Library/Caches` e `/Library/Logs` não foram exercitados pelos testes do projeto.
- **O auxiliar trabalha a partir do que ele mesmo encontra nas pastas,** não da lista que você viu na tela. Ele remove o que essas duas pastas contêm naquele momento, o que pode diferir um pouco da prévia.
- **Touch ID ou senha, quem decide é o macOS,** não o MacCleaner.
- **Precisa do app instalado** (`cargo xtask install`), porque é lá que o auxiliar fica.
- **Um risco conhecido e aceito:** um programa malicioso que já esteja rodando na sua conta de usuário poderia tentar trocar o auxiliar no instante entre a conferência e a execução. Isso é considerado aceitável para uso pessoal. Os detalhes estão na [política de segurança](../security.md#privileged-helper) (em inglês).

### Desinstalar um app que precisa de administrador

Um app que a App Store ou um instalador colocou no Mac pertence ao sistema, e o que ele deixou em `/Library` também. Remover isso funciona do mesmo jeito: o macOS autentica você e depois roda o auxiliar uma vez.

- **O auxiliar recebe qual é o app, pelo identificador, e quais itens.** Então ele mesmo encontra o app e o que pertence a ele, e só remove um item quando a busca dele também o encontrou. Um item que não é do app interrompe tudo antes de qualquer coisa ser tocada.
- **Ele recusa um app que está aberto,** e um identificador que dois apps instalados dividem.
- **Esses itens são removidos de vez.** O auxiliar roda como administrador e não alcança a sua Lixeira.
- **Se você fechar o diálogo de autenticação, nada é removido,** nem a parte que não precisa de administrador. Essa parte só começa depois que a parte de administrador deu certo.

## O que não foi testado

Dito com clareza, para você decidir o quanto de cuidado tomar:

- O diálogo de autenticação do macOS e uma limpeza de sistema real, do começo ao fim (veja acima).
- Um arquivo que esteja realmente só no iCloud.
- Abrir o app com "Reduzir movimento" ligado nos Ajustes do Sistema.
- Remover dados pessoais reais pela interface. Isso foi exercitado apenas em pastas de teste.
- Versões do macOS diferentes da 27, e Macs com Intel.
- Desinstalar um app de verdade pela interface. A busca do que pertence a um app foi conferida com apps reais, sem remover nada; a remoção rodou em pastas de teste, e a parte de administrador dela só no modo de simulação do auxiliar.
- Remover qualquer coisa de um Docker de verdade. A leitura do que o Docker guarda foi conferida em um Docker real; a remoção rodou contra um programa substituto, que responde como o Docker.
- Se o Docker Desktop devolve ao macOS o espaço liberado.

## Limites conhecidos

- **Caminhos protegidos pelo macOS não podem ser limpos.** A Proteção da Integridade do Sistema (SIP) guarda alguns arquivos até de um administrador. Eles aparecem como deixados como estavam.
- **Algumas áreas do sistema não são limpas:** os snapshots locais do Time Machine, `/private/var/log` e `/private/var/tmp`.
- **Fotos dentro da biblioteca do app Fotos ficam de fora.** Para essas, use o app Fotos.
- **O espaço realmente liberado pode ser menor que o estimado.** O formato de disco do Mac (APFS) pode guardar duas cópias de um arquivo como uma só, e pode manter versões antigas em snapshots. Remover um arquivo cujo conteúdo ainda é usado em outro lugar libera menos que o tamanho dele.
- **Itens na Lixeira continuam ocupando espaço** até você esvaziá-la.
- **O desinstalador encontra o que leva o nome do app.** Dados guardados com outro nome ficam. E um app que declare o identificador de outro app teria os itens desse outro listados como seus, então leia a lista antes de confirmar.
- **O Docker é consultado só pelo programa dele.** Outros motores de container não são procurados.

## Se algo der errado

1. **Olhe na Lixeira.** Arquivos pessoais removidos pelo MacCleaner estão lá. Clique com o botão direito e escolha **Colocar de Volta**.
2. **Leia o arquivo de histórico** para ver exatamente o que foi removido e quando. O painel mostra as entradas mais recentes.
3. **Abra uma issue** em [github.com/renatoasterio/MacCleaner/issues](https://github.com/renatoasterio/MacCleaner/issues) contando o que aconteceu. Se parecer um problema de segurança, siga o [SECURITY.md](../../SECURITY.md).

## Para quem quer ver o código

As regras acima são impostas no código e cobertas por testes:

- Regras de caminho: [`protected_paths.rs`](../../crates/maccleaner-domain/src/deletion/protected_paths.rs)
- O único caminho do código que apaga: [`delete_items.rs`](../../crates/maccleaner-application/src/use_cases/delete_items.rs)
- O auxiliar que limpa as áreas de sistema: [`apps/privileged-helper/src/main.rs`](../../apps/privileged-helper/src/main.rs)
- Como um item é ligado a um app: [`leftover_matcher.rs`](../../crates/maccleaner-domain/src/applications/leftover_matcher.rs)
- O que é pedido ao Docker: [`docker_cli_container_engine.rs`](../../crates/maccleaner-infrastructure/src/process/docker_cli_container_engine.rs)
- Como o app pede ao macOS que o rode: [`osascript_privileged_cleaner.rs`](../../crates/maccleaner-infrastructure/src/privilege/osascript_privileged_cleaner.rs)
- A política completa para quem contribui (em inglês): [docs/security.md](../security.md)
